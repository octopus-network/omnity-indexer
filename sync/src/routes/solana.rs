use crate::entity::{sea_orm_active_enums::TicketStatus, ticket};
use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, CallError, TicketId};
use candid::CandidType;
use candid::{Decode, Encode};
use core::fmt;
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::error::Error;
use std::str;

pub const SOLANA_ROUTE_CHAIN_ID: &str = "eSolana";

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
	Finalized,
	Pending,
	TxFailed { e: TxError },
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub struct TxError {
	pub block_hash: String,
	pub signature: String,
	pub error: String,
}
impl fmt::Display for TxError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"TxError: block_hash={}, signature={}, error={}",
			self.block_hash, self.signature, self.error
		)
	}
}
impl std::error::Error for TxError {}
impl TryFrom<anyhow::Error> for TxError {
	type Error = anyhow::Error;

	fn try_from(e: anyhow::Error) -> Result<Self, Self::Error> {
		if let Some(tx_error) = e.downcast_ref::<TxError>() {
			Ok(TxError {
				block_hash: tx_error.block_hash.to_owned(),
				signature: tx_error.signature.to_owned(),
				error: tx_error.error.to_owned(),
			})
		} else {
			Err(e)
		}
	}
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
	pub ticket_id: TicketId,
	pub associated_account: String,
	pub amount: u64,
	pub token_mint: String,
	pub status: TxStatus,
	pub signature: Option<String>,
	pub retry_4_building: u64,
	pub retry_4_status: u64,
}

pub async fn sync_ticket_status_from_solana_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	info!("Syncing release token status from Solana ... ");
	let unconfirmed_tickets =
		Query::get_unconfirmed_tickets(db, SOLANA_ROUTE_CHAIN_ID.to_owned()).await?;
	for unconfirmed_ticket in unconfirmed_tickets {
		ticket_status_from_solana_route(db, unconfirmed_ticket).await?;
	}
	Ok(())
}

pub async fn ticket_status_from_solana_route(
	db: &DbConn,
	ticket: ticket::Model,
) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_ROUTES_SOLANA_CANISTER_ID",
		|agent, canister_id| async move {
			let args = Encode!(&ticket.ticket_id.clone())?;
			let ret = agent
				.query(&canister_id, "mint_token_req")
				.with_arg(args)
				.call()
				.await?;

			if let Ok(mint_token_req) = Decode!(&ret, Result<MintTokenRequest, CallError>)? {
				info!(
					"Solana Mint Token Status: {:?} ",
					mint_token_req.clone().status
				);

				match mint_token_req.status {
					TxStatus::Finalized => {
						Mutation::update_ticket(
							db,
							ticket.clone(),
							Some(TicketStatus::Finalized),
							Some(mint_token_req.signature),
							None,
							None,
							None,
							None,
						)
						.await?;
					}
					TxStatus::Pending => {
						info!("{:?} is Unknown in Solana", ticket.clone())
					}
					TxStatus::TxFailed { e } => {
						info!("Solana error: {:?}  ", e)
					}
				}
			}
			Ok(())
		},
	)
	.await
}
