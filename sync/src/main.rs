use anyhow::anyhow;
use dotenvy::dotenv;
use log::LevelFilter;
use log4rs::{
	append::console::ConsoleAppender,
	config::{Appender, Root},
};
use omnity_indexer_sync::{tasks::*, utils::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	let stdout = ConsoleAppender::builder().build();
	let config = log4rs::config::Config::builder()
		.appender(Appender::builder().build("stdout", Box::new(stdout)))
		.build(Root::builder().appender("stdout").build(LevelFilter::Info))
		.unwrap();
	log4rs::init_config(config).unwrap();

	// if let Err(e) = log4rs::init_file("./log4rs.yaml", Default::default()) {
	// 	eprintln!("init log failed: {}", e);
	// 	std::process::exit(1);
	// }

	let db_url = std::env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL is not found"))?;
	let db = Database::new(db_url.clone()).await;

	let task = std::env::var("TASK")?;
	if task == "removedb" {
		execute_rm_db_tasks(db.get_connection()).await;
	} else if task == "task1800" {
		execute_tasks_1800(db.get_connection()).await;
	} else if task == "task8" {
		execute_tasks_8(db.get_connection()).await;
	} else if task == "task600" {
		execute_tasks_600(db.get_connection()).await;
	} else if task == "task30" {
		execute_tasks_30(db.get_connection()).await;
	} else if task == "task60" {
		execute_tasks_60(db.get_connection()).await;
	} else if task == "task18000" {
		execute_tasks_18000(db.get_connection()).await;
	}

	Ok(())
}
