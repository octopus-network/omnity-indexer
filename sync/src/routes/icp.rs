use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::service::{Mutation, Query};
use crate::{token_ledger_id_on_chain, with_omnity_canister, Arg};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const ROUTE_CHAIN_ID: &str = "eICP";

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
	Finalized { block_index: u64 },
	Unknown,
}

pub async fn sync_all_icp_token_ledger_id_on_chain(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_ROUTES_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			for token in Query::get_all_tokens(db).await? {
				let token_ledger = Arg::TokId(token.clone().token_id)
					.query_method(
						agent.clone(),
						canister_id,
						"get_token_ledger",
						"Syncing token ledger id from icp route ...",
						"Token ledger id from icp route result: ",
						None,
						None,
						"Option<Principal>",
					)
					.await?
					.convert_to_canister_id();
				if let Some(ledger_id) = token_ledger {
					let mut token_ledger_id = serde_json::to_string(&ledger_id).unwrap();
					token_ledger_id.replace_range(0..1, "");
					token_ledger_id.replace_range((token_ledger_id.len() - 1).., "");

					let token_ledger_id_on_chain_model = token_ledger_id_on_chain::Model::new(
						ROUTE_CHAIN_ID.to_owned(),
						token.clone().token_id,
						token_ledger_id,
					);
					// Save to the database
					let token_ledger_id_on_chain = Mutation::save_all_token_ledger_id_on_chain(
						db,
						token_ledger_id_on_chain_model,
					)
					.await?;

					info!(
						"Token {:?} in Chain id({:?})' Canister id is {:?}",
						token_ledger_id_on_chain.token_id,
						token_ledger_id_on_chain.chain_id,
						token_ledger_id_on_chain.contract_id
					);
				}
			}

			Ok(())
		},
	)
	.await
}

pub async fn sync_ticket_status_from_icp_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_ROUTES_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			//1: get ticket that dest is icp route and status is waiting for comformation by dst
			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, ROUTE_CHAIN_ID.to_owned()).await?;

			//2: get mint_token_status by ticket id
			for unconfirmed_ticket in unconfirmed_tickets {
				let mint_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"mint_token_status",
						"Syncing mint token status from icp route ...",
						"Mint token status from icp route result: ",
						None,
						None,
						"MintTokenStatus",
					)
					.await?
					.convert_to_mint_token_status();

				match mint_token_status {
					MintTokenStatus::Unknown => {
						info!(
							"Ticket id({:?}) mint token status {:?}",
							unconfirmed_ticket.ticket_id,
							MintTokenStatus::Unknown
						);
					}
					MintTokenStatus::Finalized { block_index } => {
						let tx_hash = match Query::get_token_ledger_id_on_chain_by_id(
							db,
							ROUTE_CHAIN_ID.to_owned(),
							unconfirmed_ticket.clone().token,
						)
						.await?
						{
							Some(rep) => rep.contract_id + "-" + &block_index.to_string(),
							None => block_index.to_string(),
						};

						//3: update ticket status to finalized
						let ticket_model = Mutation::update_ticket_status_n_txhash(
							db,
							unconfirmed_ticket.clone(),
							TicketStatus::Finalized,
							Some(tx_hash),
						)
						.await?;

						info!(
							"Ticket id({:?}) status:{:?} and finalized on block {:?}",
							ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
						);
					}
				}
			}
			Ok(())
		},
	)
	.await
}
