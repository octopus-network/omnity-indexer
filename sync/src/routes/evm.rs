use crate::entity::{sea_orm_active_enums::TicketStatus, ticket};
use crate::routes::MintTokenStatus;
use crate::service::{Mutation, Query};
use crate::{token_ledger_id_on_chain, with_omnity_canister, Arg, ChainId};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct EvmRoute {
	pub canister: &'static str,
	pub chain: ChainId,
}

pub struct EvmRoutes {
	routes: Vec<EvmRoute>,
}

impl EvmRoutes {
	pub fn new() -> Self {
		Self {
			routes: vec![
				EvmRoute {
					canister: "BEVM_CHAIN_ID",
					chain: "bevm".to_owned(),
				},
				EvmRoute {
					canister: "BITLAYER_CHAIN_ID",
					chain: "Bitlayer".to_owned(),
				},
				EvmRoute {
					canister: "XLAYER_CHAIN_ID",
					chain: "X Layer".to_owned(),
				},
				EvmRoute {
					canister: "BSQUARE_CHAIN_ID",
					chain: "B² Network".to_owned(),
				},
				EvmRoute {
					canister: "MERLIN_CHAIN_ID",
					chain: "Merlin".to_owned(),
				},
				EvmRoute {
					canister: "BOB_CHAIN_ID",
					chain: "Bob".to_owned(),
				},
				EvmRoute {
					canister: "ROOTSTOCK_CHAIN_ID",
					chain: "RootStock".to_owned(),
				},
				EvmRoute {
					canister: "BITFINITY_CHAIN_ID",
					chain: "Bitfinity".to_owned(),
				},
				EvmRoute {
					canister: "AILAYER_CHAIN_ID",
					chain: "AILayer".to_owned(),
				},
				EvmRoute {
					canister: "EVM_CANISTER_ID",
					chain: "Ethereum".to_owned(),
				},
				EvmRoute {
					canister: "CORE_CANISTER_ID",
					chain: "Core".to_owned(),
				},
			],
		}
	}
}

pub async fn sync_all_token_ledger_id_from_evm_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	let evm_routes = EvmRoutes::new();

	for evm_route in evm_routes.routes.iter() {
		sync_all_evm_token_ledger_id_on_chain(db, evm_route.canister, evm_route.chain.clone())
			.await?;
	}
	Ok(())
}

pub async fn sync_all_tickets_status_from_evm_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	info!("Syncing release token status from evm route ... ");
	let evm_routes = EvmRoutes::new();

	for evm_route in evm_routes.routes.iter() {
		if let (Ok(unconfirmed_tickets), Ok(deleted_unconfirmed_tickets)) = (
			Query::get_unconfirmed_tickets(db, evm_route.chain.clone()).await,
			Query::get_unconfirmed_deleted_tickets(db, evm_route.chain.clone()).await,
		) {
			for unconfirmed_ticket in unconfirmed_tickets {
				sync_ticket_status_from_evm_route(
					db,
					evm_route.canister,
					evm_route.chain.clone(),
					unconfirmed_ticket,
				)
				.await?;
			}

			for deleted_unconfirmed_ticket in deleted_unconfirmed_tickets {
				let _unconfirmed_ticket =
					ticket::Model::from_deleted_ticket(deleted_unconfirmed_ticket);
				sync_ticket_status_from_evm_route(
					db,
					evm_route.canister,
					evm_route.chain.clone(),
					_unconfirmed_ticket,
				)
				.await?;
			}
		}
	}
	Ok(())
}

async fn sync_all_evm_token_ledger_id_on_chain(
	db: &DbConn,
	canister: &str,
	chain: ChainId,
) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(canister, |agent, canister_id| async move {
		let token_ledgers = Arg::V(Vec::<u8>::new())
			.query_method(
				agent.clone(),
				canister_id,
				"get_token_list",
				"Syncing token ledger id from evm routes ...",
				"Token ledger id from evm routes result: ",
				None,
				None,
				"Vec<TokenResp>",
			)
			.await?
			.convert_to_vec_token_resp();
		for token_resp in token_ledgers {
			if let Some(evm_contract) = &token_resp.evm_contract {
				let token_ledger_id_on_chain_model = token_ledger_id_on_chain::Model::new(
					chain.clone(),
					token_resp.token_id,
					evm_contract.to_owned(),
				);
				// Save to the database
				let token_ledger_id_on_chain =
					Mutation::save_all_token_ledger_id_on_chain(db, token_ledger_id_on_chain_model)
						.await?;

				info!(
					"Token {:?} in Chain id({:?})' Contract id is {:?}",
					token_ledger_id_on_chain.token_id,
					token_ledger_id_on_chain.chain_id,
					token_ledger_id_on_chain.contract_id
				);
			}
		}

		Ok(())
	})
	.await
}

async fn sync_ticket_status_from_evm_route(
	db: &DbConn,
	canister: &str,
	_chain: ChainId,
	ticket: ticket::Model,
) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(canister, |agent, canister_id| async move {
		let mint_evm_token_status = Arg::TI(ticket.ticket_id.clone())
			.query_method(
				agent.clone(),
				canister_id,
				"mint_token_status",
				"Syncing mint token status from evm route ...",
				"Mint token status from evm route result: ",
				None,
				None,
				"MintTokenStatus",
			)
			.await?
			.convert_to_mint_token_status();

		if let MintTokenStatus::Finalized { tx_hash } = mint_evm_token_status {
			if let Ok(ticket_model) = Mutation::update_ticket(
				db,
				ticket.clone(),
				Some(TicketStatus::Finalized),
				Some(Some(tx_hash.clone())),
				None,
				None,
				None,
				None,
			)
			.await
			{
				info!(
					"evm id({:?}) status:{:?} and its hash is {:?} ",
					ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
				);
			} else if let Ok(d_ticket_model) = Mutation::update_deleted_ticket_statu_and_tx_hash(
				db,
				ticket.into(),
				Some(tx_hash),
				TicketStatus::Finalized,
			)
			.await
			{
				info!(
					"Deleted ticket id({:?}) status:{:?} and finalized on block {:?}",
					d_ticket_model.ticket_id, d_ticket_model.status, d_ticket_model.tx_hash
				);
			}
		}
		Ok(())
	})
	.await
}
