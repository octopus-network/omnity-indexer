use crate::evm;
use crate::hub::{
	CHAIN_SYNC_INTERVAL, TICKET_SYNC_INTERVAL, TOKEN_ON_CHAIN_SYNC_INTERVAL, TOKEN_SYNC_INTERVAL,
};
use crate::{customs::bitcoin, hub, routes::icp};
use futures::Future;
use log::error;
use sea_orm::DbConn;
use std::{error::Error, sync::Arc, time::Duration};

pub fn spawn_sync_task<F, Fut>(
	db_conn: Arc<DbConn>,
	interval: u64,
	sync_fn: F,
) -> tokio::task::JoinHandle<()>
where
	F: Fn(Arc<DbConn>) -> Fut + Send + Sync + 'static,
	Fut: Future<Output = Result<(), Box<dyn Error>>> + Send + 'static,
{
	tokio::spawn(async move {
		let mut interval = tokio::time::interval(Duration::from_secs(interval));
		loop {
			sync_fn(db_conn.clone()).await.unwrap_or_else(|e| {
				error!("sync task error: {}", e);
			});
			interval.tick().await;
		}
	})
}

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

	let sync_tokens_on_chains_from_huh = spawn_sync_task(
		db_conn.clone(),
		TOKEN_ON_CHAIN_SYNC_INTERVAL,
		|db_conn| async move { hub::sync_tokens_on_chains(&db_conn).await },
	);

	let sync_all_tickets_status_from_evm_route_from_evm = spawn_sync_task(
		db_conn.clone(),
		TICKET_SYNC_INTERVAL,
		|db_conn| async move { evm::sync_all_tickets_status_from_evm_route(&db_conn).await },
	);

	let _ = tokio::join!(
		sync_chains_task,
		sync_tokens_task,
		sync_tickets_task,
		sync_ticket_status_from_bitcoin,
		sync_ticket_status_from_icp,
		sync_tokens_on_chains_from_huh,
		sync_all_tickets_status_from_evm_route_from_evm
	);
}
