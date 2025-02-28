use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, Arg};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const SOLANA_CUSTOM_CHAIN_ID: &str = "Solana";

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolanaCustomReleaseTokenStatus {
	Unknown,
	Pending,
	Submitted(String),
	Finalized(String),
}

pub async fn sync_ticket_status_from_solana_custom(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_SOLANA_CANISTER_ID",
		|agent, canister_id| async move {
			info!("Syncing release token status from solana custom ... ");

			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, SOLANA_CUSTOM_CHAIN_ID.to_owned()).await?;

			for unconfirmed_ticket in unconfirmed_tickets {
				let release_solana_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"release_token_status",
						"Syncing mint token status from solana custom: ",
						"Release solana custom token status result: ",
						None,
						None,
						"SolanaCustomRelaseTokenStatus",
					)
					.await?
					.convert_to_release_solann_custom_token_status();

				if let SolanaCustomReleaseTokenStatus::Finalized(tx_hash) =
					release_solana_token_status
				{
					let ticket_model = Mutation::update_ticket(
						db,
						unconfirmed_ticket.clone(),
						Some(crate::entity::sea_orm_active_enums::TicketStatus::Finalized),
						Some(Some(tx_hash)),
						None,
						None,
						None,
						None,
					)
					.await?;

					info!(
						"solana custom ticket id({:?}) finally status:{:?} and its hash is {:?} ",
						ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
					);
				}
			}

			Ok(())
		},
	)
	.await
}
