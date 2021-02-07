use serenity::{
	client::Context,
	framework::standard::{
		Args,
		CommandResult,
		macros::command,
	},
	model::{
		channel::Message,
		id::{
			RoleId,
			ChannelId,
		},
	},
};
use yttrium_key_base::environment::{Environment, events};
use crate::types::*;
use crate::match_engine;
use crate::utilities;
use crate::databases;

#[command]
async fn execute(context: &Context, message: &Message, args: Args) -> CommandResult {
	let data = context.data.read().await;
	let keys = data.get::<KeyList>().unwrap();
	//Placeholder manager
	let pool = data.get::<Database>().unwrap();
	let db_manager = databases::SqlDatabaseManager::new(message.guild_id.unwrap(), pool);
	let environment = Environment::new(events::EventType::Default, message.guild_id.unwrap(), &context, db_manager);
	let output = yttrium::interpret_string(String::from(args.rest()), keys, environment).await;
	match output {
		Ok(result) => {
			let channel = if let Some(id) = result.result.target { id } else { message.channel_id };
			utilities::send_result(channel, &context, result).await;
		}
		Err(error) => {
			message.channel_id.say(&context.http, format!("{:#?}", error)).await.unwrap();
		}
	}
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
			let guild_id = message.guild_id.unwrap().to_string();
			let lock = context.data.read().await;
			let db = lock.get::<Database>().unwrap();
			sqlx::query!("REPLACE INTO triggers VALUES (?, ?, ?)", trigger, code, guild_id).execute(db).await.unwrap();
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
	let guild_id = message.guild_id.unwrap().to_string();
	let query = sqlx::query!("DELETE FROM triggers WHERE trigger = ? AND guild_id = ?", trigger, guild_id);
	let lock = context.data.read().await;
	let db = lock.get::<Database>().unwrap();
	match query.execute(db).await.unwrap().rows_affected() {
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
	let guild_id = message.guild_id.unwrap().to_string();
	let query = sqlx::query!("SELECT code FROM triggers WHERE trigger = ? AND guild_id = ?", trigger, guild_id);
	let lock = context.data.read().await;
	let db = lock.get::<Database>().unwrap();
	match query.fetch_optional(db).await {
		Ok(Some(result)) => {
			let code = result.code;
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
			let guild_id = message.guild_id.unwrap().to_string();
			let lock = context.data.read().await;
			let db = lock.get::<Database>().unwrap();
			let query = sqlx::query!("REPLACE INTO events VALUES (?, ?, ?)", event, guild_id, code);
			query.execute(db).await.unwrap();
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
	let guild_id = message.guild_id.unwrap().to_string();
	let lock = context.data.read().await;
	let db = lock.get::<Database>().unwrap();
	let query = sqlx::query!("DELETE FROM events WHERE event = ? AND guild_id = ?", event, guild_id);
	match query.execute(db).await.unwrap().rows_affected() {
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
	let guild_id = message.guild_id.unwrap().to_string();
	let lock = context.data.read().await;
	let db = lock.get::<Database>().unwrap();
	let query = sqlx::query!("SELECT code FROM events WHERE event = ? AND guild_id = ?", event, guild_id);
	match query.fetch_optional(db).await {
		Ok(Some(result)) => {
			let code = result.code;
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

#[command]
async fn prefix(context: &Context, message: &Message, args: Args) -> CommandResult {
	match args.parse::<String>() {
		Ok(new_prefix) => {
			let guild_id = message.guild_id.unwrap().to_string();
			let lock = context.data.read().await;
			let db = lock.get::<Database>().unwrap();
			if utilities::set_guild_prefix(&guild_id, &new_prefix, db).await {
				message.channel_id.say(&context.http, format!("Your prefix has been updated to `{}`", new_prefix)).await.unwrap();
			} else {
				message.channel_id.say(&context.http, "Could not update the prefix").await.unwrap();
			}
		}
		Err(_) => {
			let guild_id = message.guild_id.unwrap().to_string();
			let lock = context.data.read().await;
			let db = lock.get::<Database>().unwrap();
			let old_prefix = utilities::get_guild_prefix(&guild_id, db).await;
			message.channel_id.say(&context.http, format!("Your current prefix is: `{}`", old_prefix)).await.unwrap();
		}
	}
	return Ok(());
}

#[command]
async fn admin(context: &Context, message: &Message, args: Args) -> CommandResult {
	let new_role_id;
	if message.mention_roles.is_empty() {
		match args.parse::<RoleId>() {
			Ok(role) => {
				new_role_id = Some(role.to_string());
			}
			Err(_) => {
				let possible_role_name = args.rest();
				let guild = message.guild(&context.cache).await.unwrap();
				match guild.role_by_name(possible_role_name) {
					Some(role) => {
						new_role_id = Some(role.id.to_string());
					}
					None => {
						new_role_id = None;
					}
				}
			}
		}
	} else {
		new_role_id = Some(message.mention_roles[0].to_string());
	}
	let lock = context.data.read().await;
	let db = lock.get::<Database>().unwrap();
	let result = utilities::set_guild_admin(&message.guild_id.unwrap().to_string(), new_role_id, db).await;
	std::mem::drop(lock);
	if result {
		message.channel_id.say(&context.http, "Your admin role has been updated").await.unwrap();
	} else {
		message.channel_id.say(&context.http, "The update has failed").await.unwrap();
	}
	return Ok(());
}

#[command]
async fn error_channel(context: &Context, message: &Message, args: Args) -> CommandResult {
	let new_channel_id;
	if message.mention_channels.is_empty() {
		match args.parse::<ChannelId>() {
			Ok(channel) => {
				new_channel_id = Some(channel.to_string());
			}
			Err(_) => {
				let possible_role_name = args.rest();
				let guild = message.guild(&context.cache).await.unwrap();
				match guild.channel_id_from_name(&context, possible_role_name).await {
					Some(role) => {
						new_channel_id = Some(role.to_string());
					}
					None => {
						new_channel_id = None;
					}
				}
			}
		}
	} else {
		new_channel_id = Some(message.mention_channels[0].id.to_string());
	}
	let lock = context.data.read().await;
	let db = lock.get::<Database>().unwrap();
	let result = utilities::set_guild_error_channel(&message.guild_id.unwrap().to_string(), new_channel_id, db).await;
	drop(lock);
	if result {
		message.channel_id.say(&context.http, "Your error channel has been updated").await.unwrap();
	} else {
		message.channel_id.say(&context.http, "The update has failed").await.unwrap();
	}
	return Ok(());
}