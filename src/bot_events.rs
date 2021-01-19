use serenity::client::EventHandler;
use serenity::async_trait;
use yttrium_key_base::environment::{
	Environment,
	events,
};
use crate::types::*;
use crate::databases::*;

async fn get_event_code(event_name: &str, guild_id: &str, pool: &sqlx::SqlitePool) -> Option<String> {
	let query = sqlx::query!("SELECT code FROM events WHERE event = ? AND guild_id = ?", event_name, guild_id);
	match query.fetch_optional(pool).await {
		Ok(Some(result)) => {
			return Some(result.code);
		}
		Ok(None) => {
			return None;
		}
		Err(error) => {
			eprintln!("get_event_code: DB error with event: `{}` on guild `{}`: `{}`", event_name, guild_id, error);
			return None;
		}
	}
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn channel_create(&self, context: serenity::client::Context, channel: &serenity::model::channel::GuildChannel) {
		let lock = context.data.read().await;
		let db = lock.get::<DB>().unwrap();
		if let Some(code) = get_event_code("ChannelCreate", &channel.guild_id.to_string(), db).await {
			let db_manager = SQLDatabaseManager::new(channel.guild_id, db);
			let event_info = events::EventType::ChannelCreate(events::ChannelCreateEventInfo::new(channel.id));
			let environment = Environment::new(event_info, channel.guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			match output {
				Ok(output) => {
					match output.warnings {
						Some(warns) => {
							let message = format!("ChannelCreate event had the following warnings: ```{:#?}```\n{}", warns, output.result.message);
							output.result.target.say(&context.http, message).await.unwrap();
						}
						None => {
							output.result.target.say(&context.http, output.result.message).await.unwrap();
						}
					}
				}
				Err(error) => {
					unimplemented!("Error in channel_create: `{:#?}`", error);
				}
			}
		}
	}

	async fn channel_delete(&self, context: serenity::client::Context, channel: &serenity::model::channel::GuildChannel) {
		let lock = context.data.read().await;
		let db = lock.get::<DB>().unwrap();
		if let Some(code) = get_event_code("ChannelDelete", &channel.guild_id.to_string(), db).await {
			let db_manager = SQLDatabaseManager::new(channel.guild_id, db);
			let event_info = events::EventType::ChannelDelete(events::ChannelDeleteEventInfo::new(channel.id));
			let environment = Environment::new(event_info, channel.guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			match output {
				Ok(output) => {
					match output.warnings {
						Some(warns) => {
							let message = format!("ChannelCreate event had the following warnings: ```{:#?}```\n{}", warns, output.result.message);
							output.result.target.say(&context.http, message).await.unwrap();
						}
						None => {
							output.result.target.say(&context.http, output.result.message).await.unwrap();
						}
					}
				}
				Err(error) => {
					unimplemented!("Error in channel_create: `{:#?}`", error);
				}
			}
		}
	}

	async fn channel_update(&self, context: serenity::client::Context, _old: Option<serenity::model::channel::Channel>, channel: serenity::model::channel::Channel) {
		let channel = channel.guild().unwrap();
		let lock = context.data.read().await;
		let db = lock.get::<DB>().unwrap();
		if let Some(code) = get_event_code("ChannelUpdate", &channel.guild_id.to_string(), db).await {
			let db_manager = SQLDatabaseManager::new(channel.guild_id, db);
			let event_info = events::EventType::ChannelUpdate(events::ChannelUpdateEventInfo::new(channel.id));
			let environment = Environment::new(event_info, channel.guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			match output {
				Ok(output) => {
					match output.warnings {
						Some(warns) => {
							let message = format!("ChannelCreate event had the following warnings: ```{:#?}```\n{}", warns, output.result.message);
							output.result.target.say(&context.http, message).await.unwrap();
						}
						None => {
							output.result.target.say(&context.http, output.result.message).await.unwrap();
						}
					}
				}
				Err(error) => {
					unimplemented!("Error in channel_create: `{:#?}`", error);
				}
			}
		}
	}
	

	async fn guild_member_addition(&self, _ctx: serenity::client::Context, _guild_id: serenity::model::id::GuildId, _new_member: serenity::model::guild::Member) {}

	async fn guild_member_removal(&self, _ctx: serenity::client::Context, _guild_id: serenity::model::id::GuildId, _user: serenity::model::prelude::User, _member_data_if_available: Option<serenity::model::guild::Member>) {}

	async fn guild_member_update(&self, _ctx: serenity::client::Context, _old_if_available: Option<serenity::model::guild::Member>, _new: serenity::model::guild::Member) {}

	async fn guild_role_create(&self, _ctx: serenity::client::Context, _guild_id: serenity::model::id::GuildId, _new: serenity::model::guild::Role) {}

	async fn guild_role_delete(&self, _ctx: serenity::client::Context, _guild_id: serenity::model::id::GuildId, _removed_role_id: serenity::model::id::RoleId, _removed_role_data_if_available: Option<serenity::model::guild::Role>) {}

	async fn guild_role_update(&self, _ctx: serenity::client::Context, _guild_id: serenity::model::id::GuildId, _old_data_if_available: Option<serenity::model::guild::Role>, _new: serenity::model::guild::Role) {}

	async fn guild_update(&self, _ctx: serenity::client::Context, _old_data_if_available: Option<serenity::model::guild::Guild>, _new_but_incomplete: serenity::model::guild::PartialGuild) {}

	async fn reaction_add(&self, _ctx: serenity::client::Context, _add_reaction: serenity::model::channel::Reaction) {}

	async fn reaction_remove(&self, _ctx: serenity::client::Context, _removed_reaction: serenity::model::channel::Reaction) {}

	async fn voice_state_update(&self, _ctx: serenity::client::Context, _: Option<serenity::model::id::GuildId>, _old: Option<serenity::model::prelude::VoiceState>, _new: serenity::model::prelude::VoiceState) {}
}