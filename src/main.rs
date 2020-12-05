use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serenity::{
	prelude::RwLock,
	client::Context,
	model::channel::Message,
	framework::standard::{
		Args,
		CommandResult,
		macros::{group, command},
	}
};
use yttrium_key_base::environment::{Environment, events};

#[group]
#[commands(execute, add)]
struct General;

#[command]
async fn execute(context: &Context, message: &Message, args: Args) -> CommandResult {
	let keys = yttrium::key_loader::load_keys();
	//Placeholder manager
	let db_manager = Box::from(yttrium_key_base::databases::JSONDatabaseManager::new_from_json("{}", "757679825661198347"));
	let environment = Environment::new(events::EventType::Default, message.guild_id.unwrap(), &context, db_manager);
	let output = yttrium::interpret_string(String::from(args.rest()), &keys, environment);
	message.channel_id.say(&context.http, format!("{:#?}", output)).await.unwrap();
	return Ok(());
}

#[command]
async fn add(context: &Context, message: &Message, args: Args) -> CommandResult {
	let code = String::from(args.rest());
	let keys = yttrium::key_loader::load_keys();
	match yttrium::tree_creator::create_ars_tree(code, &keys) {
		Ok(_) => {
			message.channel_id.say(&context.http, "Trigger added").await.unwrap();
		}
		Err(error) => {
			message.channel_id.say(&context.http, format!("The string is invalid: {:#?}", error)).await.unwrap();
		}
	}
	return Ok(());
}

#[tokio::main]
async fn main() {
	let config_file = "./config.json5";
	let input = std::fs::read_to_string(config_file).expect("Could not read the config file");
	let bot_config = json5::from_str::<Config>(&input).expect("The config file is invalid");
	let framework = serenity::framework::StandardFramework::new().configure(|config| {
		return config.prefix(&bot_config.prefix);
	}).group(&GENERAL_GROUP);
	let mut client = serenity::Client::builder(&bot_config.token).framework(framework).await.unwrap();
	client.data.write().await.insert::<Config>(Arc::new(RwLock::new(bot_config)));
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