// use clap::{Parser, Subcommand};
use dotenvy::dotenv;
// use log::info;
use omnity_indexer_sync::{tasks::execute_sync_tasks, utils::*};
// use std::env;
use anyhow::anyhow;
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
};

// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// struct Cli {
// 	#[arg(short, long)]
// 	config: Option<String>,

// 	#[command(subcommand)]
// 	command: Option<Commands>,
// }

// #[derive(Subcommand, Debug)]
// enum Commands {
// 	/// Use config
// 	Config,
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();

	// let cli = Cli::parse();

	// let exe_path = env::current_exe()?;
	// let exe_dir = exe_path
	// 	.parent()
	// 	.unwrap()
	// 	.parent()
	// 	.unwrap()
	// 	.parent()
	// 	.unwrap();
	// let default_config_filename = "config.toml";
	// let default_config_path = exe_dir.join(default_config_filename);

	// // If the user didn't specify a config, use the default
	// let config_path = cli
	// 	.config
	// 	.unwrap_or_else(|| default_config_path.to_str().unwrap().to_string());

	// let settings = Settings::new(&config_path)?;

	// set_config(settings);

	// Initial log
	// let log_config = read_config(|c| c.log_config.to_owned());

	// if let Err(e) = log4rs::init_file("log4rs.yaml", Default::default()) {
	// 	eprintln!("init log failed: {}", e);
	// 	std::process::exit(1);
	// }

	let stdout = ConsoleAppender::builder().build();
	let config = log4rs::config::Config::builder()
	.appender(Appender::builder().build("stdout", Box::new(stdout)))
	.build(Root::builder().appender("stdout").build(LevelFilter::Info))
	.unwrap();
	log4rs::init_config(config).unwrap();

	// let db_url = match std::env::var("DATABASE_URL") {
	// 	Ok(url) => {
	// 		info!("Getting database url from env var: {}", url);
	// 		url
	// 	}
	// 	Err(_) => {
	// 		let url = read_config(|c| c.get("DATABASE_URL"))?;
	// 		info!("Getting database url from config var: {url:?}");
	// 		url
	// 	}
	// };
	let db_url = std::env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL is not found"))?;
	let db = Database::new(db_url.clone()).await;
	execute_sync_tasks(db.get_connection()).await;

	Ok(())
}
