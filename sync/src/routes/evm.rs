use crate::entity::{sea_orm_active_enums::TicketStatus, ticket};
use crate::service::{Mutation, Query};
use crate::{token_ledger_id_on_chain, with_omnity_canister, Arg, ChainId};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct EvmRoutes {
	pub canister: &'static str,
	pub chain: ChainId,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintEvmTokenStatus {
	Finalized { tx_hash: String },
	Unknown,
}

pub async fn sync_all_token_ledger_id_from_evm_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	let evm_routes = vec![
		EvmRoutes {
			canister: "BEVM_CHAIN_ID",
			chain: "bevm".to_owned(),
		},
		EvmRoutes {
			canister: "BITLAYER_CHAIN_ID",
			chain: "Bitlayer".to_owned(),
		},
		EvmRoutes {
			canister: "XLAYER_CHAIN_ID",
			chain: "X Layer".to_owned(),
		},
		EvmRoutes {
			canister: "BSQUARE_CHAIN_ID",
			chain: "B² Network".to_owned(),
		},
		EvmRoutes {
			canister: "MERLIN_CHAIN_ID",
			chain: "Merlin".to_owned(),
		},
		EvmRoutes {
			canister: "BOB_CHAIN_ID",
			chain: "Bob".to_owned(),
		},
	];

	for evm_route in evm_routes.iter() {
		let _ =
			sync_all_evm_token_ledger_id_on_chain(db, evm_route.canister, evm_route.chain.clone())
				.await;
	}
	Ok(())
}

pub async fn sync_all_tickets_status_from_evm_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	let evm_routes = vec![
		EvmRoutes {
			canister: "BEVM_CHAIN_ID",
			chain: "bevm".to_owned(),
		},
		EvmRoutes {
			canister: "BITLAYER_CHAIN_ID",
			chain: "Bitlayer".to_owned(),
		},
		EvmRoutes {
			canister: "XLAYER_CHAIN_ID",
			chain: "X Layer".to_owned(),
		},
		EvmRoutes {
			canister: "BSQUARE_CHAIN_ID",
			chain: "B² Network".to_owned(),
		},
		EvmRoutes {
			canister: "MERLIN_CHAIN_ID",
			chain: "Merlin".to_owned(),
		},
		EvmRoutes {
			canister: "BOB_CHAIN_ID",
			chain: "Bob".to_owned(),
		},
	];

	for evm_route in evm_routes.iter() {
		let unconfirmed_tickets =
			Query::get_unconfirmed_tickets(db, evm_route.chain.clone()).await?;
		for unconfirmed_ticket in unconfirmed_tickets {
			let _ = sync_ticket_status_from_evm_route(
				db,
				evm_route.canister,
				evm_route.chain.clone(),
				unconfirmed_ticket,
			)
			.await;
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
	chain: ChainId,
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
				"MintEvmTokenStatus",
			)
			.await?
			.convert_to_mint_evm_token_status();

		match mint_evm_token_status {
			MintEvmTokenStatus::Unknown => {
				info!(
					"Ticket id({:?}) from {:?} mint evm token status {:?}",
					ticket.ticket_id,
					chain.clone(),
					MintEvmTokenStatus::Unknown
				);
			}
			MintEvmTokenStatus::Finalized { tx_hash } => {
				let ticket_model = Mutation::update_ticket_status_n_txhash(
					db,
					ticket.clone(),
					TicketStatus::Finalized,
					Some(tx_hash),
				)
				.await?;

				info!(
					"Ticket id({:?}) status:{:?} and its hash is {:?} ",
					ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
				);
			}
		}
		Ok(())
	})
	.await
}
