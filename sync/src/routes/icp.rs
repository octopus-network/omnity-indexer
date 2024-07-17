use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::service::{Mutation, Query};
use crate::{types::TicketId, with_omnity_canister, Arg};
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

//This function only used for mock test
pub async fn mock_finalized_mint_token(
	ticket_id: TicketId,
	block_index: u64,
) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_ROUTES_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			let _ = Arg::TI(ticket_id)
				.query_method(
					agent.clone(),
					canister_id,
					"mock_finalized_mint_token",
					"Mock finalized mint token on icp route ...",
					"Mock finalized mint token on icp route ret: ",
					Some(block_index),
					None,
					"()",
				)
				.await?;

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
						//3: update ticket status to finalized
						let ticket_model = Mutation::update_ticket_status(
							db,
							unconfirmed_ticket.clone(),
							TicketStatus::Finalized,
						)
						.await?;
						let index_ticket_model = Mutation::update_tikcet_tx_hash(
							db,
							unconfirmed_ticket.clone(),
							block_index.to_string(),
						)
						.await?;

						info!(
							"Ticket id({:?}) status:{:?} and finalized on block {:?}",
							ticket_model.ticket_id, ticket_model.status, index_ticket_model.tx_hash
						);
					}
				}
			}

			Ok(())
		},
	)
	.await
}
