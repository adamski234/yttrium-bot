use serenity::client::EventHandler;
use serenity::async_trait;
use sqlx::Row;
use yttrium_key_base::environment::{
	Environment,
	events,
};
use crate::types::*;
use crate::databases::*;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn channel_create(&self, context: serenity::client::Context, channel: &serenity::model::channel::GuildChannel) {
		let query = format!("SELECT code FROM events WHERE event = \"ChannelCreate\" AND guild_id = \"{}\"", channel.guild_id);
		let lock = context.data.read().await;
		let db = lock.get::<DB>().unwrap();
		match sqlx::query(&query).fetch_optional(db).await {
			Ok(Some(result)) => {
				let code: String = result.get("code");
				let db_manager = SQLDatabaseManager::new(channel.guild_id, db);
				let event_info = events::EventType::ChannelCreate(events::ChannelCreateEventInfo::new(channel.id));
				let environment = Environment::new(event_info, channel.guild_id, &context, db_manager);
				let keys = lock.get::<KeyList>().unwrap();
				let output = yttrium::interpret_string(code, keys, environment);
				match output {
					Ok(output) => {
						//This should use environment.target
						println!("{:#?}", output);
					}
					Err(error) => {
						unimplemented!("Error in channel_create: `{:#?}`", error);
					}
				}
			}
			Ok(None) => {
				return;
			}
			Err(error) => {
				eprintln!("DB error: `{}`", error);
				return;
			}
		}
	}

	async fn channel_delete(&self, _ctx: serenity::client::Context, _channel: &serenity::model::channel::GuildChannel) {}

	async fn channel_update(&self, _ctx: serenity::client::Context, _old: Option<serenity::model::channel::Channel>, _new: serenity::model::channel::Channel) {}

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