use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serenity::prelude::*;
use serenity::async_trait;
use yttrium_key_base::environment::{Environment, events};

struct EventHandler;

#[async_trait]
impl serenity::client::EventHandler for EventHandler {
	async fn message(&self, context: Context, message: serenity::model::channel::Message) {
		let prefix = context.data.read().await.get::<Config>().unwrap().read().await.prefix.clone();
		if message.content.starts_with(&prefix) {
			let split = message.content.split_whitespace().collect::<Vec<&str>>();
			if split[0].ends_with("execute") {
				let keys = yttrium::key_loader::load_keys("");
				//Placeholder manager
				let db_manager = Box::from(yttrium_key_base::databases::JSONDatabaseManager::new_from_json("{}", "757679825661198347"));
				let environment = Environment::new(events::EventType::Default, message.guild_id.unwrap(), &context, db_manager);
				message.channel_id.say(&context.http, format!("{:#?}", yttrium::interpret_string(split[1..].join(" "), &keys.keys, environment))).await.unwrap();
			}
		}
	}
}

#[tokio::main]
async fn main() {
	let config_file = "./config.json5";
	let input = std::fs::read_to_string(config_file).expect("Could not read the config file");
	let config = json5::from_str::<Config>(&input).expect("The config file is invalid");
	let framework = serenity::framework::StandardFramework::default();
	let mut client = serenity::Client::builder(&config.token).event_handler(EventHandler).framework(framework).await.unwrap();
	client.data.write().await.insert::<Config>(Arc::new(RwLock::new(config)));
	client.start().await.unwrap();
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
struct Config {
	token: String,
	prefix: String,
}

impl serenity::prelude::TypeMapKey for Config {
	type Value = Arc<RwLock<Config>>;
}