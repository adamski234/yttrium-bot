use serenity::framework::standard::macros::hook;

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