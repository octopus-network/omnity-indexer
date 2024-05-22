use crate::spawn_sync_task;
use crate::{customs::bitcoin, hub, routes::icp};
use sea_orm::DbConn;

use std::sync::Arc;

use crate::hub::CHAIN_SYNC_INTERVAL;
use crate::hub::TICKET_SYNC_INTERVAL;
use crate::hub::TOKEN_SYNC_INTERVAL;

pub async fn execute_sync_tasks(db_conn: Arc<DbConn>) {
    let sync_chains_task =
        spawn_sync_task(db_conn.clone(), CHAIN_SYNC_INTERVAL, |db_conn| async move {
            hub::sync_chains(&db_conn).await
        });

    let sync_tokens_task =
        spawn_sync_task(db_conn.clone(), TOKEN_SYNC_INTERVAL, |db_conn| async move {
            hub::sync_tokens(&db_conn).await
        });

    let sync_tickets_task = spawn_sync_task(
        db_conn.clone(),
        TICKET_SYNC_INTERVAL,
        |db_conn| async move { hub::sync_tickets(&db_conn).await },
    );

    let sync_ticket_status_from_bitcoin = spawn_sync_task(
        db_conn.clone(),
        TICKET_SYNC_INTERVAL,
        |db_conn| async move { bitcoin::sync_ticket_status_from_bitcoin(&db_conn).await },
    );

    let sync_ticket_status_from_icp = spawn_sync_task(
        db_conn.clone(),
        TICKET_SYNC_INTERVAL,
        |db_conn| async move { icp::sync_ticket_status_from_icp_route(&db_conn).await },
    );
    let _ = tokio::join!(
        sync_chains_task,
        sync_tokens_task,
        sync_tickets_task,
        sync_ticket_status_from_bitcoin,
        sync_ticket_status_from_icp
    );
}
