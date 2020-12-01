use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serenity::prelude::*;
use serenity::async_trait;

struct EventHandler;

#[async_trait]
impl serenity::client::EventHandler for EventHandler {
	async fn message(&self, context: Context, message: serenity::model::channel::Message) {
		if message.content == format!("{}ping", context.data.read().await.get::<Config>().unwrap().read().await.prefix) {
			message.reply(&context.http, "Lmao no").await.unwrap();
		};
		//if message.content == config.pre
	}
}

fn main() {
	let config_file = "./config.json5";
	let input = std::fs::read_to_string(config_file).expect("Could not read the config file");
	let config = json5::from_str::<Config>(&input).expect("The config file is invalid");
	let mut client = futures::executor::block_on(serenity::Client::builder(&config.token).event_handler(EventHandler)).unwrap();
	//client.type_map_insert::<Config>(config);
	futures::executor::block_on(client.data.write()).insert::<Config>(Arc::new(RwLock::new(config)));
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
struct Config {
	token: String,
	prefix: String,
}

impl serenity::prelude::TypeMapKey for Config {
	type Value = Arc<RwLock<Config>>;
}