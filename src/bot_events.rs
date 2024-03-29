use serenity::{
	client::EventHandler,
	async_trait,
	prelude::Context,
};
use yttrium_key_base::environment::{
	Environment,
	events,
};
use yttrium::{
	ResultAndWarnings,
	errors_and_warns::Error
};
use crate::utilities;
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

async fn send_output(event_name: &str, context: &Context, output: Result<ResultAndWarnings<'_, SqlDatabaseManager, SqlDatabase>, Error>) {
	match output {
		Ok(output) => {
			utilities::send_result(context, output).await;
		}
		Err(error) => {
			unimplemented!("Error in {}: `{:#?}`", event_name, error);
		}
	}
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn channel_create(&self, context: serenity::client::Context, channel: &serenity::model::channel::GuildChannel) {
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("ChannelCreate", &channel.guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(channel.guild_id, db);
			let event_info = events::EventType::ChannelCreate(events::ChannelCreateEventInfo::new(channel.id));
			let environment = Environment::new(event_info, channel.guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("ChannelCreate", &context, output).await;
		}
	}

	async fn channel_delete(&self, context: serenity::client::Context, channel: &serenity::model::channel::GuildChannel) {
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("ChannelDelete", &channel.guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(channel.guild_id, db);
			let event_info = events::EventType::ChannelDelete(events::ChannelDeleteEventInfo::new(channel.id));
			let environment = Environment::new(event_info, channel.guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("ChannelDelete", &context, output).await;
		}
	}

	async fn channel_update(&self, context: serenity::client::Context, _old: Option<serenity::model::channel::Channel>, channel: serenity::model::channel::Channel) {
		let channel = channel.guild().unwrap();
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("ChannelUpdate", &channel.guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(channel.guild_id, db);
			let event_info = events::EventType::ChannelUpdate(events::ChannelUpdateEventInfo::new(channel.id));
			let environment = Environment::new(event_info, channel.guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("ChannelUpdate", &context, output).await;
		}
	}
	

	async fn guild_member_addition(&self, context: serenity::client::Context, guild_id: serenity::model::id::GuildId, new_member: serenity::model::guild::Member) {
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("MemberJoin", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::MemberJoin(events::MemberJoinEventInfo::new(new_member.user.id));
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("MemberJoin", &context, output).await;
		}
	}

	async fn guild_member_removal(&self, context: serenity::client::Context, guild_id: serenity::model::id::GuildId, user: serenity::model::prelude::User, _member_data_if_available: Option<serenity::model::guild::Member>) {
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("MemberLeave", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::MemberLeave(events::MemberLeaveEventInfo::new(user.id));
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("MemberLeave", &context, output).await;
		}
	}

	async fn guild_member_update(&self, context: serenity::client::Context, _old_if_available: Option<serenity::model::guild::Member>, member: serenity::model::guild::Member) {
		let guild_id = member.guild_id;
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("MemberUpdate", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::MemberUpdate(events::MemberUpdateEventInfo::new(member.user.id));
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("MemberUpdate", &context, output).await;
		}
	}

	async fn guild_role_create(&self, context: serenity::client::Context, guild_id: serenity::model::id::GuildId, new: serenity::model::guild::Role) {
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("RoleCreate", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::RoleCreate(events::RoleCreateEventInfo::new(new.id));
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("RoleCreate", &context, output).await;
		}
	}

	async fn guild_role_delete(&self, context: serenity::client::Context, guild_id: serenity::model::id::GuildId, removed_role_id: serenity::model::id::RoleId, _removed_role_data_if_available: Option<serenity::model::guild::Role>) {
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("RoleDelete", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::RoleDelete(events::RoleDeleteEventInfo::new(removed_role_id));
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("RoleDelete", &context, output).await;
		}
	}

	async fn guild_role_update(&self, context: serenity::client::Context, guild_id: serenity::model::id::GuildId, _old_data_if_available: Option<serenity::model::guild::Role>, new: serenity::model::guild::Role) {
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("RoleUpdate", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::RoleUpdate(events::RoleUpdateEventInfo::new(new.id));
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("RoleUpdate", &context, output).await;
		}
	}

	async fn guild_update(&self, context: serenity::client::Context, _old_data_if_available: Option<serenity::model::guild::Guild>, new: serenity::model::guild::PartialGuild) {
		let guild_id = new.id;
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("GuildUpdate", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::GuildUpdate(events::GuildUpdateEventInfo::new());
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("GuildUpdate", &context, output).await;
		}
	}

	async fn reaction_add(&self, context: serenity::client::Context, reaction: serenity::model::channel::Reaction) {
		let guild_id = reaction.guild_id.unwrap();
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("ReactionAdd", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::ReactionAdd(events::ReactionAddEventInfo::new(reaction.channel_id, reaction.message_id, reaction.user_id.unwrap(), reaction.emoji));
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("ReactionAdd", &context, output).await;
		}
	}

	async fn reaction_remove(&self, context: serenity::client::Context, reaction: serenity::model::channel::Reaction) {
		let guild_id = reaction.guild_id.unwrap();
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("ReactionRemove", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::ReactionRemove(events::ReactionRemoveEventInfo::new(reaction.channel_id, reaction.message_id, reaction.user_id.unwrap(), reaction.emoji));
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("ReactionRemove", &context, output).await;
		}
	}

	async fn voice_state_update(&self, context: serenity::client::Context, guild_id: Option<serenity::model::id::GuildId>, _old: Option<serenity::model::prelude::VoiceState>, new: serenity::model::prelude::VoiceState) {
		let guild_id = guild_id.unwrap();
		let lock = context.data.read().await;
		let db = lock.get::<Database>().unwrap();
		if let Some(code) = get_event_code("VoiceUpdate", &guild_id.to_string(), db).await {
			let db_manager = SqlDatabaseManager::new(guild_id, db);
			let event_info = events::EventType::VoiceUpdate(events::VoiceUpdateEventInfo::new(new.channel_id.unwrap(), new.user_id));
			let environment = Environment::new(event_info, guild_id, &context, db_manager);
			let keys = lock.get::<KeyList>().unwrap();
			let output = yttrium::interpret_string(code, keys, environment).await;
			send_output("VoiceUpdate", &context, output).await;
		}
	}
}