This is a sample bot using the [yttrium](https://github.com/adamski234/yttrium) library

It currently uses the git version of the library, not what is on crates.io, due to the WIP nature of the project. It also does not provide dynamic loading.

# How to build
Clone this repository, create a `.env` file in the root directory which contains two variables:
* `DISCORD_TOKEN` - being the token used for logging the bot into Discord
* `DATABASE_URL` - being where the SQLite database file is stored. Needs to start with `sqlite:`, like `sqlite:data.db`

Then download [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli) and execute `cargo sqlx database create` and `cargo sqlx migrate run`.  
Now you can simply execute `cargo build` and everything will download and compile.