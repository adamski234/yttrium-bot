#![feature(with_options)]

use std::{
	hint::unreachable_unchecked,
	sync::Arc,
	io::{
		Read,
		Write,
	}
};
use serde::{Deserialize, Serialize};
use serenity::{
	prelude::{RwLock, TypeMapKey},
	client::Context,
	model::channel::Message,
	framework::standard::{
		Args,
		CommandResult,
		macros::{group, command},
	}
};
use yttrium_key_base::environment::{Environment, events};
mod triggers;

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
			let mut file = std::fs::File::with_options().read(true).write(true).create(true).open(format!("./triggers/{}.json", message.guild_id.unwrap())).unwrap();
			let mut file_read = String::new();
			file.read_to_string(&mut file_read).unwrap();
			if file_read.is_empty() {
				file_read = String::from("{}");
			}
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
				yttrium::errors_and_warns::Error::InterpretationError(_) => {
					unsafe {
						unreachable_unchecked();
					}
				}
			}
		}
	}
	return Ok(());
}

#[tokio::main]
async fn main() {
	let config_file = "./config.json5";
	let input = std::fs::read_to_string(config_file).expect("Could not read the config file");
	let bot_config = json5::from_str::<Config>(&input).expect("The config file is invalid");
	let framework = serenity::framework::StandardFramework::new().configure(|config| {
		return config.prefix(&bot_config.prefix);
	}).group(&GENERAL_GROUP);
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