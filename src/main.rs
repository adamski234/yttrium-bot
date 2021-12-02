#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

mod databases;
mod match_engine;
mod utilities;
mod bot_events;
mod types;
mod commands;
use serenity::{
	client::{
		Context,
		bridge::gateway::GatewayIntents,
	},
	framework::standard::macros::{group, hook},
	model::channel::Message,
};
use yttrium_key_base::environment::Environment;
use types::*;
use commands::*;
use utilities::*;

#[group]
#[checks(is_guild_admin)]
#[commands(execute, add, remove, show, event_add, event_remove, event_show, prefix, admin, error_channel)]
struct General;

#[hook]
async fn normal_message_hook(context: &Context, message: &Message) {
	let guild_id = message.guild_id.unwrap().to_string();
	let lock = context.data.read().await;
	let db = lock.get::<Database>().unwrap();
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
			let pool = data.get::<Database>().unwrap();
			let db_manager = databases::SqlDatabaseManager::new(message.guild_id.unwrap(), pool);
			let event_info = yttrium_key_base::environment::events::MessageEventInfo::new(message.channel_id, message.id, message.author.id, parameter, trigger);
			let event = yttrium_key_base::environment::events::EventType::Message(event_info);
			let environment = Environment::new(event, message.guild_id.unwrap(), context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let result = yttrium::interpret_string(code.clone(), keys, environment).await;
			match result {
				Ok(result) => {
					utilities::send_result(&context, result).await;
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
	let framework = serenity::framework::StandardFramework::new().configure(|config| {
		return config.dynamic_prefix(|context, message| Box::pin(async move {
			let lock = context.data.read().await;
			let db = lock.get::<Database>().unwrap();
			return Some(utilities::get_guild_prefix(&message.guild_id.unwrap().to_string(), db).await);
		})).prefix("");
	}).group(&GENERAL_GROUP).normal_message(normal_message_hook);
	let mut client = serenity::Client::builder(env!("DISCORD_TOKEN")).intents(GatewayIntents::all()).framework(framework).event_handler(bot_events::Handler).await.unwrap();
	let mut bot_data = client.data.write().await;
	let data = sqlx::SqlitePool::connect(env!("DATABASE_URL")).await.unwrap();
	bot_data.insert::<Database>(data);
	let keys = yttrium::key_loader::load_keys();
	bot_data.insert::<KeyList>(keys);
	std::mem::drop(bot_data);
	client.start().await.unwrap();
}