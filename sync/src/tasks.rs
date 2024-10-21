use crate::customs::UPDATE_DELETED_MINT_TICKET_SYNC_INTERVAL;
use crate::hub::{
	CHAIN_SYNC_INTERVAL, TICKET_SYNC_INTERVAL, TOKEN_ON_CHAIN_SYNC_INTERVAL, TOKEN_SYNC_INTERVAL,
	TOKEN_VOLUME_SYNC_INTERVAL,
};
use crate::routes::TOKEN_LEDGER_ID_ON_CHAIN_SYNC_INTERVAL;
use crate::Delete;
use crate::{
	customs::{bitcoin, sicp},
	evm, hub,
	routes::{cosmwasm, icp, solana},
};
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
	let remove_database = async {
		let _ = Delete::remove_chains(&db_conn).await;
		let _ = Delete::remove_tokens(&db_conn).await;
		let _ = Delete::remove_tickets(&db_conn).await;
		let _ = Delete::remove_token_on_chains(&db_conn).await;
		let _ = Delete::remove_token_ledger_id_on_chain(&db_conn).await;
		let _ = Delete::remove_deleted_mint_tickets(&db_conn).await;
		let _ = Delete::remove_pending_mint_tickets(&db_conn).await;
		let _ = Delete::remove_token_volumes(&db_conn).await;
	};

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

	let sync_all_token_ledger_id_on_chain_from_icp = spawn_sync_task(
		db_conn.clone(),
		TOKEN_LEDGER_ID_ON_CHAIN_SYNC_INTERVAL,
		|db_conn| async move { icp::sync_all_icp_token_ledger_id_on_chain(&db_conn).await },
	);

	let sync_all_token_ledger_id_from_evm = spawn_sync_task(
		db_conn.clone(),
		TOKEN_LEDGER_ID_ON_CHAIN_SYNC_INTERVAL,
		|db_conn| async move { evm::sync_all_token_ledger_id_from_evm_route(&db_conn).await },
	);

	let sync_tokens_on_chains_from_hub = spawn_sync_task(
		db_conn.clone(),
		TOKEN_ON_CHAIN_SYNC_INTERVAL,
		|db_conn| async move { hub::sync_tokens_on_chains(&db_conn).await },
	);

	let sync_ticket_status_from_solana = spawn_sync_task(
		db_conn.clone(),
		TICKET_SYNC_INTERVAL,
		|db_conn| async move { solana::sync_ticket_status_from_solana_route(&db_conn).await },
	);

	let sync_ticket_status_from_bitcoin = spawn_sync_task(
		db_conn.clone(),
		TICKET_SYNC_INTERVAL,
		|db_conn| async move { bitcoin::sync_all_ticket_status_from_bitcoin(&db_conn).await },
	);

	let sync_ticket_status_from_sicp = spawn_sync_task(
		db_conn.clone(),
		TICKET_SYNC_INTERVAL,
		|db_conn| async move { sicp::sync_ticket_status_from_sicp(&db_conn).await },
	);

	let sync_ticket_status_from_eicp = spawn_sync_task(
		db_conn.clone(),
		TICKET_SYNC_INTERVAL,
		|db_conn| async move { icp::sync_ticket_status_from_icp_route(&db_conn).await },
	);

	let sync_all_tickets_status_from_evm = spawn_sync_task(
		db_conn.clone(),
		TICKET_SYNC_INTERVAL,
		|db_conn| async move { evm::sync_all_tickets_status_from_evm_route(&db_conn).await },
	);

	let sync_all_tickets_status_from_cosmwasm = spawn_sync_task(
		db_conn.clone(),
		TICKET_SYNC_INTERVAL,
		|db_conn| async move { cosmwasm::sync_all_tickets_status_from_cosmwasm_route(&db_conn).await },
	);

	let update_sender_tickets_from_hub = spawn_sync_task(
		db_conn.clone(),
		TICKET_SYNC_INTERVAL,
		|db_conn| async move { hub::update_sender(&db_conn).await },
	);

	let update_mint_tickets_from_btc = spawn_sync_task(
		db_conn.clone(),
		TICKET_SYNC_INTERVAL,
		|db_conn| async move { bitcoin::update_mint_tickets(&db_conn).await },
	);

	let update_deleted_mint_tickets_from_btc = spawn_sync_task(
		db_conn.clone(),
		UPDATE_DELETED_MINT_TICKET_SYNC_INTERVAL,
		|db_conn| async move { bitcoin::update_deleted_mint_tickets(&db_conn).await },
	);

	let update_total_volumes_from_hub = spawn_sync_task(
		db_conn.clone(),
		TOKEN_VOLUME_SYNC_INTERVAL,
		|db_conn| async move { hub::update_volume(&db_conn).await },
	);

	let _ = tokio::join!(
		remove_database,
		sync_chains_task,
		sync_tokens_task,
		sync_tickets_task,
		sync_all_token_ledger_id_on_chain_from_icp,
		sync_all_token_ledger_id_from_evm,
		sync_tokens_on_chains_from_hub,
		sync_ticket_status_from_solana,
		sync_ticket_status_from_bitcoin,
		sync_ticket_status_from_sicp,
		sync_ticket_status_from_eicp,
		sync_all_tickets_status_from_evm,
		sync_all_tickets_status_from_cosmwasm,
		update_sender_tickets_from_hub,
		update_mint_tickets_from_btc,
		update_deleted_mint_tickets_from_btc,
		update_total_volumes_from_hub,
	);
}
