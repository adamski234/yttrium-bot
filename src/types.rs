use serenity::prelude::TypeMapKey;
use crate::databases::{
	SQLDatabase,
	SQLDatabaseManager,
};

pub struct DB;

impl TypeMapKey for DB {
	type Value = sqlx::sqlite::SqlitePool;
}

pub struct KeyList;

impl TypeMapKey for KeyList {
	type Value = std::collections::HashMap<String, Box<dyn yttrium_key_base::Key<SQLDatabaseManager, SQLDatabase> + Sync + Send>>;
}