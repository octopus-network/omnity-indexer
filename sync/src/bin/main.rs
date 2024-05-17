use std::env;

use clap::Parser;

use log4rs;

use omnity_indexer_sync::hub::CHAIN_SYNC_INTERVAL;
use omnity_indexer_sync::hub::TICKET_SYNC_INTERVAL;
use omnity_indexer_sync::hub::TOKEN_SYNC_INTERVAL;
#[cfg(debug_assertions)]
// use dotenvy::dotenv;
use omnity_indexer_sync::{customs::bitcoin, hub, routes::icp, utils::*};
use sea_orm::DbConn;
use std::time::Duration;
use tokio::task;
use tokio::time;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// the path to the config file
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // #[cfg(debug_assertions)]
    // dotenv().ok();

    let args = Args::parse();
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    println!("exe_dir: {:?}", exe_dir);

    let default_config_filename = "config.toml";
    let default_config_path = exe_dir.join(default_config_filename);

    // If the user didn't specify a config, use the default
    let config_path = args
        .config
        .unwrap_or_else(|| default_config_path.to_str().unwrap().to_string());

    let settings = Settings::new(&config_path)?;
    println!("settings: {:?}", settings);
    set_config(settings);
    // init log
    let log_config = read_config(|c| c.log_config.to_owned());
    if let Err(e) = log4rs::init_file(log_config, Default::default()) {
        eprintln!("init log failed: {}", e);
        std::process::exit(1);
    }
    // init database
    let db_url = read_config(|c| c.database_url.to_owned());
    let db = Database::new(db_url).await;

    execute_sync_tasks(&db.get_connection()).await;
    Ok(())
}

async fn execute_sync_tasks(db_conn: &DbConn) {
    let sync_chains_task = task::spawn({
        let db_conn_chains = db_conn.clone();
        async move {
            let mut interval = time::interval(Duration::from_secs(CHAIN_SYNC_INTERVAL));
            loop {
                hub::sync_chains(&db_conn_chains).await;
                interval.tick().await;
            }
        }
    });
    let sync_tokens_task = task::spawn({
        let db_conn_tokens = db_conn.clone();
        async move {
            let mut interval = time::interval(Duration::from_secs(TOKEN_SYNC_INTERVAL));
            loop {
                hub::sync_tokens(&db_conn_tokens).await;
                interval.tick().await;
            }
        }
    });
    let sync_tickets_task = task::spawn({
        let db_conn_tickets = db_conn.clone();
        async move {
            let mut interval = time::interval(Duration::from_secs(TICKET_SYNC_INTERVAL));
            loop {
                hub::sync_tickets(&db_conn_tickets).await;
                interval.tick().await;
            }
        }
    });
    let sync_pending_tickets_from_bitcion = task::spawn({
        let db_conn_pending_ticket = db_conn.clone();
        async move {
            let mut interval = time::interval(Duration::from_secs(TICKET_SYNC_INTERVAL));
            loop {
                bitcoin::sync_pending_tickets_from_bitcoin(&db_conn_pending_ticket).await;
                interval.tick().await;
            }
        }
    });
    let sync_ticket_status_from_bitcoin = task::spawn({
        let db_conn_ticket_status = db_conn.clone();

        async move {
            let mut interval = time::interval(Duration::from_secs(TICKET_SYNC_INTERVAL));
            loop {
                bitcoin::sync_ticket_status_from_bitcoin(&db_conn_ticket_status).await;
                interval.tick().await;
            }
        }
    });
    let sync_ticket_status_from_icp = task::spawn({
        let db_conn_ticket_status = db_conn.clone();
        async move {
            let mut interval = time::interval(Duration::from_secs(TICKET_SYNC_INTERVAL));
            loop {
                icp::sync_ticket_status_from_icp_route(&db_conn_ticket_status).await;
                interval.tick().await;
            }
        }
    });
    let _ = tokio::join!(
        sync_chains_task,
        sync_tokens_task,
        sync_tickets_task,
        sync_pending_tickets_from_bitcion,
        sync_ticket_status_from_bitcoin,
        sync_ticket_status_from_icp
    );
}
