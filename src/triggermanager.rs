#![allow(dead_code)] //Remove after finishing

use std::collections::HashMap;
use serenity::model::id::GuildId;

pub struct TriggerManager {
	file: std::fs::File,
	guilds: HashMap<GuildId, GuildTriggerManager>,
}

impl TriggerManager {
	pub fn new(file: std::fs::File) -> Self {
		return Self {
			file: file,
			guilds: HashMap::new(),
		};
	}
	pub fn get_guild(&self, guild_id: &GuildId) -> Option<&GuildTriggerManager> {
		return self.guilds.get(guild_id);
	}
	pub fn save(&mut self) {
		//
	}
}

pub struct GuildTriggerManager {
	guild_id: GuildId,
	triggers: HashMap<String, String>,
}

impl GuildTriggerManager {
	pub fn new(guild_id: GuildId) -> Self {
		return Self {
			guild_id: guild_id.clone(),
			triggers: HashMap::new(),
		};
	}
}