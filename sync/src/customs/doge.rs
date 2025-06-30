use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, Arg};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const DOGECOIN_CUSTOM_CHAIN_ID: &str = "Dogecoin";

#[derive(Serialize, Deserialize, Debug, Clone, candid::CandidType)]
pub enum DogecoinReleaseTokenStatus {
	Unknown,
	Pending,
	Signing,
	Sending(String),
	Submitted(String),
	Confirmed(String),
}

pub async fn sync_ticket_status_from_doge(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_DOGECOIN_CANISTER_ID",
		|agent, canister_id| async move {
			info!("Syncing release token status from doge custom ... ");

			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, DOGECOIN_CUSTOM_CHAIN_ID.to_owned()).await?;

			for unconfirmed_ticket in unconfirmed_tickets {
				let release_doge_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"release_token_status",
						"Syncing mint token status from doge custom: ",
						"  ",
						None,
						None,
						"DogecoinCustomRelaseTokenStatus",
					)
					.await?
					.convert_to_release_dogecoin_token_status();

				if let DogecoinReleaseTokenStatus::Confirmed(tx_hash) = release_doge_token_status {
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
						"dogecoin custom ticket id({:?}) finally status:{:?} and its hash is {:?} ",
						ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
					);
				}
			}

			Ok(())
		},
	)
	.await
}
