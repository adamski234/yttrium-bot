use sqlx::Row;

pub struct SQLDatabase {
	guild_id: serenity::model::id::GuildId,
	name: String,
	pool: sqlx::SqlitePool,
}

impl SQLDatabase {
	pub fn new(guild_id: serenity::model::id::GuildId, pool: sqlx::SqlitePool, name: String) -> Self {
		return Self { guild_id, pool, name };
	}
}

pub struct SQLDatabaseManager {
	guild_id: serenity::model::id::GuildId,
	pool: sqlx::SqlitePool,
}

impl SQLDatabaseManager {
	pub fn new(guild_id: serenity::model::id::GuildId, pool: &sqlx::SqlitePool) -> Self {
		return Self {
			guild_id: guild_id,
			pool: pool.clone(),
		};
	}
}

impl yttrium_key_base::databases::Database for SQLDatabase {
    fn get_key(&self, name: &str) -> Option<yttrium_key_base::databases::StringOrArray> {
		let query = format!("SELECT key_content FROM databases WHERE name = {} AND guild_id = {} AND key_name = {}", self.name, self.guild_id, name);
		let result = futures::executor::block_on(sqlx::query(&query).fetch_one(&self.pool));
		match result {
			Ok(result) => {
				let content = result.get::<String, &str>("key_content");
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
		let query = format!("REPLACE INTO databases VALUES ({}, {}, {}, {})", self.name, self.guild_id, name, to_insert);
		futures::executor::block_on(sqlx::query(&query).execute(&self.pool)).unwrap();
    }

    fn remove_key(&mut self, name: &str) {
        let query = format!("DELETE FROM databases WHERE name = {} AND guild_id = {} AND key_name = {}", self.name, self.guild_id, name);
		futures::executor::block_on(sqlx::query(&query).execute(&self.pool)).unwrap();
    }

    fn key_exists(&self, name: &str) -> bool {
		let query = format!("SELECT name FROM databases WHERE name = {} AND guild_id = {} AND key_name = {}", self.name, self.guild_id, name);
		let result = futures::executor::block_on(sqlx::query(&query).fetch_optional(&self.pool));
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

impl yttrium_key_base::databases::DatabaseManager<SQLDatabase> for SQLDatabaseManager {
	fn get_database(&mut self, name: &str) -> SQLDatabase {
		return SQLDatabase::new(self.guild_id.clone(), self.pool.clone(), String::from(name))
	}

	fn remove_database(&mut self, name: &str) {
		let query = format!("DELETE FROM databases WHERE name = {} AND guild_id = {}", name, self.guild_id);
		futures::executor::block_on(sqlx::query(&query).execute(&self.pool)).unwrap();
	}

	fn clear_database(&mut self, name: &str) {
		let query = format!("DELETE FROM databases WHERE name = {} AND guild_id = {}", name, self.guild_id);
		futures::executor::block_on(sqlx::query(&query).execute(&self.pool)).unwrap();
	}
}

unsafe impl Send for SQLDatabaseManager {}
unsafe impl Sync for SQLDatabaseManager {}