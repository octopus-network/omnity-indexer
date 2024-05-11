use hub::{sync_chains, sync_tickets, sync_tokens};
use omnity_indexer_sync::hub;
use omnity_indexer_sync::utils::*;

#[cfg(debug_assertions)]
use dotenvy::dotenv;

use tokio::task;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    // let db = Database::new().await;
    let db = Database::new(db_url).await;

    let sync_chains_task = task::spawn({
        let db_conn_chains = db.get_connection();
        async move {
            let mut interval = time::interval(Duration::from_secs(5));
            loop {
                sync_chains(&db_conn_chains.clone()).await;
                interval.tick().await;
            }
        }
    });
    let sync_tokens_task = task::spawn({
        let db_conn_tokens = db.get_connection();
        async move {
            let mut interval = time::interval(Duration::from_secs(5));
            loop {
                sync_tokens(&db_conn_tokens.clone()).await;
                interval.tick().await;
            }
        }
    });
    let sync_tickets_task = task::spawn({
        let db_conn_tickets = db.get_connection();
        async move {
            let mut interval = time::interval(Duration::from_secs(5));
            loop {
                sync_tickets(&db_conn_tickets.clone()).await;
                interval.tick().await;
            }
        }
    });
    let _ = tokio::join!(sync_chains_task, sync_tokens_task, sync_tickets_task);
}
