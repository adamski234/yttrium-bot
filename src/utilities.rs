use serenity::{
	model::{
		channel::Message,
		id::RoleId,
	},
	prelude::Context,
	framework::standard::{
		Args,
		CommandOptions,
		Reason,
		macros::{hook, check}
	}
};

use crate::types::*;

/// Returns a properly capitalized event name, or [None] if the original string was empty or didn't contain an event name
pub fn proper_event_name(original: &str) -> Option<&str> {
	match original.to_ascii_lowercase().as_ref() {
		"memberjoin" => {
			return Some("MemberJoin");
		}
		"memberleave" => {
			return Some("MemberLeave");
		}
		"memberupdate" => {
			return Some("MemberUpdate");
		}
		"rolecreate" => {
			return Some("RoleCreate");
		}
		"roleupdate" => {
			return Some("RoleUpdate");
		}
		"roledelete" => {
			return Some("RoleDelete");
		}
		"channelcreate" => {
			return Some("ChannelCreate");
		}
		"channeldelete" => {
			return Some("ChannelDelete");
		}
		"channelupdate" => {
			return Some("ChannelUpdate");
		}
		"guildupdate" => {
			return Some("GuildUpdate");
		}
		"voiceupdate" => {
			return Some("VoiceUpdate");
		}
		"reactionadd" => {
			return Some("ReactionAdd");
		}
		"reactionremove" => {
			return Some("ReactionRemove");
		}
		_ => {
			return None;
		}
	};
}

#[hook]
pub async fn get_guild_prefix(guild_id: &str, database: &sqlx::SqlitePool) -> String {
	let query = sqlx::query!("SELECT prefix FROM config WHERE guild_id = ?", guild_id);
	match query.fetch_optional(database).await {
		Ok(result) => {
			match result {
				Some(result) => {
					match result.prefix {
						Some(prefix) => {
							return prefix;
						}
						None => {
							return String::from(".");
						}
					}
				}
				None => {
					return String::from(".");
				}
			}
		}
		Err(error) => {
			eprintln!("get_guild_prefix: Error: `{}`", error);
			return String::new();
		}
	}
}

pub async fn set_guild_prefix(guild_id: &str, new_prefix: &str, database: &sqlx::SqlitePool) -> bool {
	let query = sqlx::query!("INSERT INTO config (guild_id, prefix) VALUES (?, ?) ON CONFLICT (guild_id) DO UPDATE SET prefix = ?", guild_id, new_prefix, new_prefix);
	let result = query.execute(database).await.unwrap();
	return result.rows_affected() == 1;
}

#[check]
pub async fn is_guild_admin(context: &Context, message: &Message, _args: &mut Args, _command_options: &CommandOptions) -> Result<(), Reason> {
	let permissions = message.member(context).await.unwrap().permissions(context).await.unwrap();
	/*if permissions.administrator() {
		return Ok(());
	}*/
	let guild_id = message.guild_id.unwrap().to_string();
	let query  = sqlx::query!("SELECT admin_role FROM config WHERE guild_id = ?", guild_id);
	let lock = context.data.read().await;
	let db = lock.get::<DB>().unwrap();
	match query.fetch_optional(db).await {
		Ok(result) => {
			match result {
				Some(result) => {
					//Some admin role was set, check if the user has it
					match result.admin_role {
						Some(admin_role) => {
							let role_id = RoleId::from(admin_role.parse::<u64>().unwrap());
							if message.member.as_ref().unwrap().roles.contains(&role_id) {
								return Ok(());
							} else {
								return Err(Reason::User(String::from("You do not have the required role")));
							}
						}
						None => {
							//No admin role set in the config, check for server management permissions
							if permissions.manage_guild() {
								return Ok(());
							} else {
								return Err(Reason::User(String::from("You do not have the Manage Guild permission")));
							}
						}
					}
				}
				None => {
					//No admin role set in the config, check for server management permissions
					if permissions.manage_guild() {
						return Ok(());
					} else {
						return Err(Reason::User(String::from("You do not have the Manage Guild permission")));
					}
				}
			}
		}
		Err(error) => {
			return Err(Reason::UserAndLog {
				user: String::from("Database error"),
				log: format!("is_guild_admin: Database error: `{}`", error),
			});
		}
	}
}