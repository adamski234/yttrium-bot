#![feature(with_options)]

use std::{
	sync::Arc,
	io::{Read, Write},
};
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
mod triggers;
mod match_engine;

#[group]
#[commands(execute, add)]
struct General;

#[command]
async fn execute(context: &Context, message: &Message, args: Args) -> CommandResult {
	let keys = yttrium::key_loader::load_keys();
	//Placeholder manager
	let db_manager = Box::from(yttrium_key_base::databases::JSONDatabaseManager::new_from_json("{}", "757679825661198347"));
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
	let keys = yttrium::key_loader::load_keys();
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
			let mut file = std::fs::File::with_options().read(true).write(true).truncate(true).create(true).open(format!("./triggers/{}.json", message.guild_id.unwrap())).unwrap();
			let mut file_read = String::new();
			file.read_to_string(&mut file_read).unwrap();
			let mut trigger_map;
			match json5::from_str::<triggers::Triggers>(&file_read) {
				Ok(map) => {
					trigger_map = map;
				}
				Err(_) => {
					trigger_map = triggers::Triggers::new();
				}
			}
			trigger_map.messages.insert(trigger, code);
			file.write_all(json5::to_string(&trigger_map).unwrap().as_bytes()).unwrap();
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
	let mut file = std::fs::File::with_options().read(true).open(format!("./triggers/{}.json", message.guild_id.unwrap())).unwrap();
	let mut file_read = String::new();
	file.read_to_string(&mut file_read).unwrap();
	let trigger_map;
	match json5::from_str::<triggers::Triggers>(&file_read) {
		Ok(map) => {
			trigger_map = map;
		}
		Err(_) => {
			trigger_map = triggers::Triggers::new();
		}
	}
	for (trigger, code) in &trigger_map.messages {
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
				let db_manager = Box::from(yttrium_key_base::databases::JSONDatabaseManager::new(&message.guild_id.unwrap().to_string()));
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