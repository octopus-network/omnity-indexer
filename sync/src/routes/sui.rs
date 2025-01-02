use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, CallError, TicketId};
use candid::{Decode, Encode};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const SUI_CHAIN_ID: &str = "eSui";

#[derive(candid::CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SuiMintTokenRequest {
	pub ticket_id: TicketId,
	pub token_id: String,
	pub recipient: String,
	pub amount: u64,
	pub status: TxStatus,
	pub digest: Option<String>,
	pub object: Option<String>,
	pub retry: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
	New,
	Pending,
	Finalized,
	TxFailed { e: String },
}

pub async fn sync_ticket_status_from_sui(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("SUI_CANISTER_ID", |agent, canister_id| async move {
		info!("Syncing release token status from sui ... ");
		let unconfirmed_tickets =
			Query::get_unconfirmed_tickets(db, SUI_CHAIN_ID.to_owned()).await?;

		for unconfirmed_ticket in unconfirmed_tickets {
			let args = Encode!(&unconfirmed_ticket.ticket_id.clone())?;
			let ret = agent
				.query(&canister_id, "mint_token_req")
				.with_arg(args)
				.call()
				.await?;

			if let Ok(mint_token_req) = Decode!(&ret, Result<SuiMintTokenRequest, CallError>)? {
				match mint_token_req.status {
					TxStatus::Finalized => {
						Mutation::update_ticket(
							db,
							unconfirmed_ticket.clone(),
							Some(TicketStatus::Finalized),
							Some(mint_token_req.digest),
							None,
							None,
							None,
							None,
						)
						.await?;
					}
					TxStatus::Pending => {
						info!("{:?} is Unknown in sui", unconfirmed_ticket.clone())
					}
					TxStatus::New => {
						info!("sui new ")
					}
					TxStatus::TxFailed { e } => {
						info!("sui error: {:?}  ", e)
					}
				}
			}
		}
		Ok(())
	})
	.await
}
