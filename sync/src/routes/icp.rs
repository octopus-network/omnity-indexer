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
			//step1: get ticket that dest is icp route chain and status is waiting for comformation
			// by dst
			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, ROUTE_CHAIN_ID.to_owned()).await?;

			//step2: get mint_token_status by ticket id
			for unconfirmed_ticket in unconfirmed_tickets {
				let mint_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"mint_token_status",
						"Syncing mint token status from icp route ...",
						" ",
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
						info!(
							"Ticket id({:?}) finalized on block {:?}",
							unconfirmed_ticket.ticket_id, block_index
						);

						//step3: update ticket status to finalized
						let ticket_modle = Mutation::update_ticket_status(
							db,
							unconfirmed_ticket,
							TicketStatus::Finalized,
						)
						.await?;
						info!(
							"Ticket id({:?}) status:{:?} ",
							ticket_modle.ticket_id, ticket_modle.status
						);
					}
				}
			}

			Ok(())
		},
	)
	.await
}

//TODO: nothing to do from icp redeem to customs
pub async fn sync_redeem_tickets(_db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_ROUTES_ICP_CANISTER_ID",
		|_agent, _canister_id| async move { Ok(()) },
	)
	.await
}
