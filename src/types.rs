use serenity::prelude::TypeMapKey;
use crate::databases::{
	SqlDatabase,
	SqlDatabaseManager,
};

pub struct Database;

impl TypeMapKey for Database {
	type Value = sqlx::sqlite::SqlitePool;
}

pub struct KeyList;

impl TypeMapKey for KeyList {
	type Value = std::collections::HashMap<String, Box<dyn yttrium_key_base::Key<SqlDatabaseManager, SqlDatabase> + Sync + Send>>;
}