use anyhow::anyhow;
use dotenvy::dotenv;
use log::LevelFilter;
use log4rs::{
	append::console::ConsoleAppender,
	config::{Appender, Root},
};
use omnity_indexer_sync::{tasks::execute_sync_tasks, utils::*};
use warp::Filter;

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

	let port: u16 = std::env::var("PORT")
		.unwrap_or_else(|_| "8080".to_string())
		.parse()
		.map_err(|_| anyhow!("PORT must be a valid number"))?;
	let health_route = warp::path!("health").map(|| warp::reply::json(&"OK"));
	tokio::spawn(async move {
		warp::serve(health_route).run(([0, 0, 0, 0], port)).await;
	});

	let db_url = std::env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL is not found"))?;
	let db = Database::new(db_url.clone()).await;
	execute_sync_tasks(db.get_connection()).await;

	Ok(())
}
