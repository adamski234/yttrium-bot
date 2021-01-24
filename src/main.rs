#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

#![feature(with_options)]
#![feature(async_closure)]

mod databases;
mod match_engine;
mod utilities;
mod bot_events;
mod types;
mod commands;
use std::sync::Arc;
use serenity::{
	client::{
		Context,
		bridge::gateway::GatewayIntents,
	},
	framework::standard::macros::{group, hook},
	model::channel::Message,
	prelude::RwLock,
};
use yttrium_key_base::environment::Environment;
use types::*;
use commands::*;

#[group]
#[commands(execute, add, remove, show, event_add, event_remove, event_show, prefix)]
struct General;

#[hook]
async fn normal_message_hook(context: &Context, message: &Message) {
	let guild_id = message.guild_id.unwrap().to_string();
	let lock = context.data.read().await;
	let db = lock.get::<DB>().unwrap();
	let query = sqlx::query!("SELECT trigger, code FROM triggers WHERE guild_id = ?", guild_id);
	let result = query.fetch_all(db).await.unwrap();
	for row in result {
		let trigger = row.trigger;
		let code = row.code;
		//Starting with nothing: starting literal
		//Starting with `&`: literal
		//Starting with `?`: regex
		let trigger_type = match_engine::MatchType::new(trigger);
			if let Some(result) = match_engine::check_match(&message.content, trigger_type) {
			let parameter = result.rest;
			let trigger = result.matched;
			let data = context.data.read().await;
			let pool = data.get::<DB>().unwrap();
			let db_manager = databases::SQLDatabaseManager::new(message.guild_id.unwrap(), pool);
			let event_info = yttrium_key_base::environment::events::MessageEventInfo::new(message.channel_id, message.id, message.author.id, parameter, trigger);
			let event = yttrium_key_base::environment::events::EventType::Message(event_info);
			let environment = Environment::new(event, message.guild_id.unwrap(), context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let result = yttrium::interpret_string(code.clone(), keys, environment).await;
			match result {
				Ok(result) => {
					message.channel_id.say(&context.http, result.result.message).await.unwrap();
				}
				Err(error) => {
					if let yttrium::errors_and_warns::Error::InterpretationError(error) = error {
						message.channel_id.say(&context.http, format!("An error happened during interpretation: `{}`", error)).await.unwrap();
					}
				}
			}
			return;
		}
	}
}

#[tokio::main]
async fn main() {
	let config_file = "./config.json5";
	let input = std::fs::read_to_string(config_file).expect("Could not read the config file");
	let bot_config = json5::from_str::<Config>(&input).expect("The config file is invalid");
	let framework = serenity::framework::StandardFramework::new().configure(|config| {
		return config.dynamic_prefix(|context, message| Box::pin(async move {
			let lock = context.data.read().await;
			let db = lock.get::<DB>().unwrap();
			return Some(utilities::get_guild_prefix(&message.guild_id.unwrap().to_string(), db).await);
		})).prefix("");
	}).group(&GENERAL_GROUP).normal_message(normal_message_hook);
	let mut client = serenity::Client::builder(&bot_config.token).intents(GatewayIntents::all()).framework(framework).event_handler(bot_events::Handler).await.unwrap();
	let mut bot_data = client.data.write().await;
	bot_data.insert::<Config>(Arc::new(RwLock::new(bot_config)));
	let data = sqlx::SqlitePool::connect(env!("DATABASE_URL")).await.unwrap();
	bot_data.insert::<DB>(data);
	let keys = yttrium::key_loader::load_keys();
	bot_data.insert::<KeyList>(keys);
	std::mem::drop(bot_data);
	client.start().await.unwrap();
}