#![feature(with_options)]

mod databases;
mod match_engine;
mod utilities;
mod bot_events;
mod types;
use std::sync::Arc;
use sqlx::{Done, Row};
use serenity::{
	prelude::RwLock,
	client::Context,
	model::channel::Message,
	framework::standard::{
		Args,
		CommandResult,
		macros::{group, command, hook},
	}
};
use yttrium_key_base::environment::{Environment, events};
use types::*;

#[group]
#[commands(execute, add, remove, show, event_add, event_remove, event_show)]
struct General;

#[command]
async fn execute(context: &Context, message: &Message, args: Args) -> CommandResult {
	let data = context.data.read().await;
	let keys = data.get::<KeyList>().unwrap();
	//Placeholder manager
	let pool = data.get::<DB>().unwrap();
	let db_manager = databases::SQLDatabaseManager::new(message.guild_id.unwrap(), pool);
	let environment = Environment::new(events::EventType::Default, message.guild_id.unwrap(), &context, db_manager);
	let output = yttrium::interpret_string(String::from(args.rest()), keys, environment).await;
	message.channel_id.say(&context.http, format!("{:#?}", output)).await.unwrap();
	return Ok(());
}

#[command]
async fn add(context: &Context, message: &Message, mut args: Args) -> CommandResult {
	let trigger = args.single_quoted::<String>().unwrap();
	args.unquoted();
	let code = String::from(args.rest());
	if code.is_empty() {
		message.channel_id.say(&context.http, "The trigger does not have a response").await.unwrap();
		return Ok(());
	}
	let data = context.data.read().await;
	let keys = data.get::<KeyList>().unwrap();
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
			let lock = context.data.read().await;
			let db = lock.get::<DB>().unwrap();
			sqlx::query(&format!("REPLACE INTO triggers VALUES (\"{}\", \"{}\", \"{}\")", trigger, code, guild_id)).execute(db).await.unwrap();
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

#[command]
async fn remove(context: &Context, message: &Message, args: Args) -> CommandResult {
	let trigger = args.parse::<String>().unwrap();
	let query = format!("DELETE FROM triggers WHERE trigger = \"{}\" AND guild_id = \"{}\"", trigger, message.guild_id.unwrap());
	let lock = context.data.read().await;
	let db = lock.get::<DB>().unwrap();
	match sqlx::query(&query).execute(db).await.unwrap().rows_affected() {
		0 => {
			message.channel_id.say(&context.http, "Trigger not found").await.unwrap();
		}
		_ => {
			message.channel_id.say(&context.http, "Trigger deleted").await.unwrap();
		}
	}
	return Ok(());
}

#[command]
async fn show(context: &Context, message: &Message, mut args: Args) -> CommandResult {
	args.quoted();
	let trigger = args.parse::<String>().unwrap();
	let query = format!("SELECT code FROM triggers WHERE trigger = \"{}\" AND guild_id = \"{}\"", trigger, message.guild_id.unwrap());
	let lock = context.data.read().await;
	let db = lock.get::<DB>().unwrap();
	match sqlx::query(&query).fetch_optional(db).await {
		Ok(Some(result)) => {
			let code = result.get::<String, &str>("code");
			let trigger_type = match_engine::MatchType::new(trigger);
			match trigger_type {
				match_engine::MatchType::Literal(_) => {
					message.channel_id.say(&context.http, format!("Trigger type: Literal```\n{}\n```", code)).await.unwrap();
				}
				match_engine::MatchType::StartingLiteral(_) => {
					message.channel_id.say(&context.http, format!("Trigger type: Starting literal\n```\n{}\n```", code)).await.unwrap();
				}
				match_engine::MatchType::Regex(_) => {
					message.channel_id.say(&context.http, format!("Trigger type: Regex\n```\n{}\n```", code)).await.unwrap();
				}
			}
		}
		Ok(None) => {
			message.channel_id.say(&context.http, "Trigger not found").await.unwrap();
		}
		Err(error) => {
			eprintln!("{}", error);
		}
	}
	return Ok(());
}

#[command]
async fn event_add(context: &Context, message: &Message, mut args: Args) -> CommandResult {
	let event;
	match args.current() {
		Some(ev) => {
			match utilities::proper_event_name(ev) {
				Some(ev) => {
					event = String::from(ev);
				}
				None => {
					message.channel_id.say(&context.http, "You need to provide a correct event type").await.unwrap();
					return Ok(());
				}
			}
		}
		None => {
			message.channel_id.say(&context.http, "You need to provide a correct event type").await.unwrap();
			return Ok(());
		}
	}
	args.advance();
	let code = String::from(args.rest());
	if code.is_empty() {
		message.channel_id.say(&context.http, "You need to provide a response to the event").await.unwrap();
		return Ok(());
	}
	let lock = context.data.read().await;
	let keys = lock.get::<KeyList>().unwrap();
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
					message.channel_id.say(&context.http, format!("Event added, but it has the following errors:\n {}", output)).await.unwrap();
				}
				None => {
					message.channel_id.say(&context.http, "Event added").await.unwrap();
				}
			}
			let guild_id = message.guild_id.unwrap();
			let lock = context.data.read().await;
			let db = lock.get::<DB>().unwrap();
			let query = format!("REPLACE INTO events VALUES (\"{}\", \"{}\", \"{}\")", event, guild_id, code);
			sqlx::query(&query).execute(db).await.unwrap();
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

#[command]
async fn event_remove(context: &Context, message: &Message, args: Args) -> CommandResult {
	let event;
	match args.current() {
		Some(ev) => {
			match utilities::proper_event_name(ev) {
				Some(ev) => {
					event = ev;
				}
				None => {
					message.channel_id.say(&context.http, "You need to provide a correct event type").await.unwrap();
					return Ok(());
				}
			}
		}
		None => {
			message.channel_id.say(&context.http, "You need to provide a correct event type").await.unwrap();
			return Ok(());
		}
	}
	let query = format!("DELETE FROM events WHERE event = \"{}\" AND guild_id = \"{}\"", event, message.guild_id.unwrap());
	let lock = context.data.read().await;
	let db = lock.get::<DB>().unwrap();
	match sqlx::query(&query).execute(db).await.unwrap().rows_affected() {
		0 => {
			message.channel_id.say(&context.http, "Event not found").await.unwrap();
		}
		_ => {
			message.channel_id.say(&context.http, "Event deleted").await.unwrap();
		}
	}
	return Ok(());
}

#[command]
async fn event_show(context: &Context, message: &Message, args: Args) -> CommandResult {
	let event;
	match args.current() {
		Some(ev) => {
			match utilities::proper_event_name(ev) {
				Some(ev) => {
					event = ev;
				}
				None => {
					message.channel_id.say(&context.http, "You need to provide a correct event type").await.unwrap();
					return Ok(());
				}
			}
		}
		None => {
			message.channel_id.say(&context.http, "You need to provide a correct event type").await.unwrap();
			return Ok(());
		}
	}
	let query = format!("SELECT code FROM events WHERE event = \"{}\" AND guild_id = \"{}\"", event, message.guild_id.unwrap());
	let lock = context.data.read().await;
	let db = lock.get::<DB>().unwrap();
	match sqlx::query(&query).fetch_optional(db).await {
		Ok(Some(result)) => {
			let code = result.get::<String, &str>("code");
			message.channel_id.say(&context.http, format!("```\n{}\n```", code)).await.unwrap();
		}
		Ok(None) => {
			message.channel_id.say(&context.http, "Event not found").await.unwrap();
		}
		Err(error) => {
			eprintln!("{}", error);
		}
	}
	return Ok(());
}

#[hook]
async fn normal_message_hook(context: &Context, message: &Message) {
	let guild_id = message.guild_id.unwrap();
	let lock = context.data.read().await;
	let db = lock.get::<DB>().unwrap();
	let query = format!("SELECT trigger, code FROM triggers WHERE guild_id = \"{}\"", guild_id);
	let result = sqlx::query(&query).fetch_all(db).await.unwrap();
	for row in result {
		let trigger: String = row.get("trigger");
		let code: String = row.get("code");
		//Starting with nothing: starting literal
		//Starting with `&`: literal
		//Starting with `?`: regex
		let trigger_type = match_engine::MatchType::new(trigger);
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
	let mut client = serenity::Client::builder(&bot_config.token).framework(framework).event_handler(bot_events::Handler).await.unwrap();
	let mut bot_data = client.data.write().await;
	bot_data.insert::<Config>(Arc::new(RwLock::new(bot_config)));
	let data = sqlx::SqlitePool::connect("./data.db").await.unwrap();
	bot_data.insert::<DB>(data);
	let keys = yttrium::key_loader::load_keys();
	bot_data.insert::<KeyList>(keys);
	std::mem::drop(bot_data);
	client.start().await.unwrap();
}