#![feature(with_options)]

mod databases;
mod match_engine;
use std::sync::Arc;
use sqlx::Row;
use serde::{Deserialize, Serialize};
use serenity::{
	prelude::{RwLock, TypeMapKey},
	client::Context,
	model::channel::Message,
	framework::standard::{
		Args,
		CommandResult,
		macros::{group, command, hook},
	}
};
use yttrium_key_base::environment::{Environment, events};
use databases::{
	SQLDatabase,
	SQLDatabaseManager,
};

#[group]
#[commands(execute, add)]
struct General;

#[command]
async fn execute(context: &Context, message: &Message, args: Args) -> CommandResult {
	let keys = yttrium::key_loader::load_keys();
	//Placeholder manager
	let data = context.data.read().await;
	let pool = data.get::<DB>().unwrap();
	let db_manager = databases::SQLDatabaseManager::new(message.guild_id.unwrap(), pool);
	let environment = Environment::new(events::EventType::Default, message.guild_id.unwrap(), &context, db_manager);
	let output = yttrium::interpret_string(String::from(args.rest()), &keys, environment);
	message.channel_id.say(&context.http, format!("{:#?}", output)).await.unwrap();
	return Ok(());
}

#[command]
async fn add(context: &Context, message: &Message, mut args: Args) -> CommandResult {
	let trigger = args.single_quoted::<String>().unwrap();
	args.unquoted();
	let code = String::from(args.rest());
	let keys = yttrium::key_loader::load_keys::<SQLDatabaseManager, SQLDatabase>();
	match yttrium::tree_creator::create_ars_tree(code.clone(), &keys) {
		Ok(tree) => {
			match tree.warnings {
				Some(warnings) => {
					let mut output = String::new();
					for warning in warnings {
						match warning {
							yttrium::errors_and_warns::Warning::UnclosedKeys => {
								output.push_str("There are unclosed keys");
							}
						}
					}
					message.channel_id.say(&context.http, format!("Trigger added, but it has the following errors:\n {}", output)).await.unwrap();
				}
				None => {
					message.channel_id.say(&context.http, "Trigger added").await.unwrap();
				}
			}
			let guild_id = message.guild_id.unwrap();
			let lock = context.data.write().await;
			let db = lock.get::<DB>().unwrap();
			sqlx::query(&format!("REPLACE INTO triggers VALUES (NULL, \"{}\", \"{}\", \"{}\")", trigger, code, guild_id)).execute(db).await.unwrap();
		}
		Err(error) => {
			match error {
				yttrium::errors_and_warns::Error::WrongAmountOfParameters => {
					message.channel_id.say(&context.http, "One of your keys has invalid amount of parameters").await.unwrap();
				}
				yttrium::errors_and_warns::Error::EmptyParameter => {
					message.channel_id.say(&context.http, "One of your keys has an empty parameter").await.unwrap();
				}
				yttrium::errors_and_warns::Error::NonexistentKey => {
					message.channel_id.say(&context.http, "One of your keys does not exist").await.unwrap();
				}
				yttrium::errors_and_warns::Error::InterpretationError(_) => {}
			}
		}
	}
	return Ok(());
}

#[hook]
async fn normal_message_hook(context: &Context, message: &Message) {
	let guild_id = message.guild_id.unwrap();
	let lock = context.data.write().await;
	let db = lock.get::<DB>().unwrap();
	let query = format!("SELECT trigger, code FROM triggers WHERE guild_id = \"{}\"", guild_id);
	let result = sqlx::query(&query).fetch_all(db).await.unwrap();
	for row in result {
		let trigger: String = row.get("trigger");
		let code: String = row.get("code");
		//Starting with nothing: starting literal
		//Starting with `&`: literal
		//Starting with `?`: regex
		let trigger_type;
		if trigger.starts_with('&') {
			trigger_type = match_engine::MatchType::Literal(String::from(trigger.trim_start_matches('&')));
		} else if trigger.starts_with('?') {
			trigger_type = match_engine::MatchType::Regex(regex::Regex::new(trigger.trim_start_matches('?')).unwrap());
		} else {
			trigger_type = match_engine::MatchType::StartingLiteral(trigger.clone());
		}
		match match_engine::check_match(&message.content, trigger_type) {
			Some(result) => {
				let parameter = result.rest;
				let trigger = result.matched;
				let data = context.data.read().await;
				let pool = data.get::<DB>().unwrap();
				let db_manager = databases::SQLDatabaseManager::new(message.guild_id.unwrap(), pool);
				let event_info = yttrium_key_base::environment::events::MessageEventInfo::new(message.channel_id, message.id, message.author.id, parameter, trigger);
				let event = yttrium_key_base::environment::events::EventType::Message(event_info);
				let environment = Environment::new(event, message.guild_id.unwrap().clone(), context, db_manager);
				let result = yttrium::interpret_string(code.clone(), &yttrium::key_loader::load_keys(), environment);
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
			None => {}
		}
	}
}

#[tokio::main]
async fn main() {
	let config_file = "./config.json5";
	let input = std::fs::read_to_string(config_file).expect("Could not read the config file");
	let bot_config = json5::from_str::<Config>(&input).expect("The config file is invalid");
	let framework = serenity::framework::StandardFramework::new().configure(|config| {
		return config.prefix(&bot_config.prefix);
	}).group(&GENERAL_GROUP).normal_message(normal_message_hook);
	let mut client = serenity::Client::builder(&bot_config.token).framework(framework).await.unwrap();
	client.data.write().await.insert::<Config>(Arc::new(RwLock::new(bot_config)));
	let data = sqlx::SqlitePool::connect("./data.db").await.unwrap();
	client.data.write().await.insert::<DB>(data);
	client.start().await.unwrap();
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
struct Config {
	token: String,
	prefix: String,
}

impl TypeMapKey for Config {
	type Value = Arc<RwLock<Config>>;
}

struct DB;

impl TypeMapKey for DB {
	type Value = sqlx::sqlite::SqlitePool;
}