pub struct SqlDatabase {
	guild_id: serenity::model::id::GuildId,
	name: String,
	pool: sqlx::SqlitePool,
}

impl SqlDatabase {
	pub fn new(guild_id: serenity::model::id::GuildId, pool: sqlx::SqlitePool, name: String) -> Self {
		return Self { guild_id, pool, name };
	}
}

pub struct SqlDatabaseManager {
	guild_id: serenity::model::id::GuildId,
	pool: sqlx::SqlitePool,
}

impl SqlDatabaseManager {
	pub fn new(guild_id: serenity::model::id::GuildId, pool: &sqlx::SqlitePool) -> Self {
		return Self {
			guild_id: guild_id,
			pool: pool.clone(),
		};
	}
}

impl yttrium_key_base::databases::Database for SqlDatabase {
    fn get_key(&self, name: &str) -> Option<yttrium_key_base::databases::StringOrArray> {
		let guild_id = self.guild_id.to_string();
		let query = sqlx::query!("SELECT key_value FROM databases WHERE name = ? AND guild_id = ? AND key_name = ?", self.name, guild_id, name);
		let result = futures::executor::block_on(query.fetch_one(&self.pool));
		match result {
			Ok(result) => {
				let content = result.key_value;
				return Some(yttrium_key_base::databases::StringOrArray::String(content));
			}
			Err(error) => {
				eprintln!("{}", error);
				return None;
			}
		}
    }

    fn write_key(&mut self, name: String, value: yttrium_key_base::databases::StringOrArray) {
		let to_insert;
		match value {
			yttrium_key_base::databases::StringOrArray::String(text) => {
				to_insert = text;
			}
			yttrium_key_base::databases::StringOrArray::Array(_array) => {
				todo!();
			}
		}
		let guild_id = self.guild_id.to_string();
		let query = sqlx::query!("REPLACE INTO databases VALUES (?, ?, ?, ?)", self.name, guild_id, name, to_insert);
		futures::executor::block_on(query.execute(&self.pool)).unwrap();
    }

    fn remove_key(&mut self, name: &str) {
		let guild_id = self.guild_id.to_string();
        let query = sqlx::query!("DELETE FROM databases WHERE name = ? AND guild_id = ? AND key_name = ?", self.name, guild_id, name);
		futures::executor::block_on(query.execute(&self.pool)).unwrap();
    }

    fn key_exists(&self, name: &str) -> bool {
		let guild_id = self.guild_id.to_string();
		let query = sqlx::query!("SELECT name FROM databases WHERE name = ? AND guild_id = ? AND key_name = ?", self.name, guild_id, name);
		let result = futures::executor::block_on(query.fetch_optional(&self.pool));
		match result {
			Ok(Some(_)) => {
				return true;
			}
			Ok(None) => {
				return false;
			}
			Err(error) => {
				eprintln!("{}", error);
				return false;
			}
		}
    }
}

impl yttrium_key_base::databases::DatabaseManager<SqlDatabase> for SqlDatabaseManager {
	fn get_database(&mut self, name: &str) -> SqlDatabase {
		return SqlDatabase::new(self.guild_id, self.pool.clone(), String::from(name))
	}

	fn remove_database(&mut self, name: &str) {
		let guild_id = self.guild_id.to_string();
		let query = sqlx::query!("DELETE FROM databases WHERE name = ? AND guild_id = ?", name, guild_id);
		futures::executor::block_on(query.execute(&self.pool)).unwrap();
	}

	fn clear_database(&mut self, name: &str) {
		let guild_id = self.guild_id.to_string();
		let query = sqlx::query!("DELETE FROM databases WHERE name = ? AND guild_id = ?", name, guild_id);
		futures::executor::block_on(query.execute(&self.pool)).unwrap();
	}
}

unsafe impl Send for SqlDatabaseManager {}
unsafe impl Sync for SqlDatabaseManager {}