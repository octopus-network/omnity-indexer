use anyhow::anyhow;
use dotenvy::dotenv;
use log::LevelFilter;
use log4rs::{
	append::console::ConsoleAppender,
	config::{Appender, Root},
};
use omnity_indexer_sync::{tasks::execute_sync_tasks, utils::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	let stdout = ConsoleAppender::builder().build();
	let config = log4rs::config::Config::builder()
		.appender(Appender::builder().build("stdout", Box::new(stdout)))
		.build(Root::builder().appender("stdout").build(LevelFilter::Info))
		.unwrap();
	log4rs::init_config(config).unwrap();

	let db_url = std::env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL is not found"))?;
	let db = Database::new(db_url.clone()).await;
	execute_sync_tasks(db.get_connection()).await;

	Ok(())
}
