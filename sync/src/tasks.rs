use crate::{customs::bitcoin, hub, routes::icp};
use sea_orm::DbConn;
use std::time::Duration;
use tokio::task;
use tokio::time;

use crate::hub::CHAIN_SYNC_INTERVAL;
use crate::hub::TICKET_SYNC_INTERVAL;
use crate::hub::TOKEN_SYNC_INTERVAL;

pub async fn execute_sync_tasks(db_conn: &DbConn) {
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
        sync_ticket_status_from_bitcoin,
        sync_ticket_status_from_icp
    );
}
