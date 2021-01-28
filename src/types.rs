use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serenity::prelude::{RwLock, TypeMapKey};
use crate::databases::{
	SQLDatabase,
	SQLDatabaseManager,
};

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct Config {
	pub token: String,
}

impl TypeMapKey for Config {
	type Value = Arc<RwLock<Config>>;
}

pub struct DB;

impl TypeMapKey for DB {
	type Value = sqlx::sqlite::SqlitePool;
}

pub struct KeyList;

impl TypeMapKey for KeyList {
	type Value = std::collections::HashMap<String, Box<dyn yttrium_key_base::Key<SQLDatabaseManager, SQLDatabase> + Sync + Send>>;
}