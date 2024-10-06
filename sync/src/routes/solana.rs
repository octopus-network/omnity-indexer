use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, Arg, TicketId};
use candid::CandidType;
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str;

pub const SOLANA_ROUTE_CHAIN_ID: &str = "eSolana";

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
	Finalized,
	Unknown,
	TxFailed { e: String },
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
	pub ticket_id: TicketId,
	pub associated_account: String,
	pub amount: u64,
	pub token_mint: String,
	pub status: TxStatus,
	pub signature: Option<String>,
	pub retry: u64,
}

impl MintTokenRequest {
	pub fn new() -> Self {
		Self {
			ticket_id: "".to_string(),
			associated_account: "".to_string(),
			amount: 0,
			token_mint: "".to_string(),
			status: TxStatus::Finalized,
			signature: Some("00000000".to_string()),
			retry: 0,
		}
	}
}

pub async fn sync_ticket_status_from_solana_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_ROUTES_SOLANA_CANISTER_ID",
		|agent, canister_id| async move {
			info!(
				"{:?} Syncing release token status from Solana ... ",
				chrono::Utc::now()
			);
			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, SOLANA_ROUTE_CHAIN_ID.to_owned()).await?;

			for unconfirmed_ticket in unconfirmed_tickets {
				let mint_token_req = Arg::TI(unconfirmed_ticket.clone().ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"mint_token_req",
						"Syncing mint token status from solana route ...",
						"Mint token status from solana route result: ",
						None,
						None,
						"MintTokenRequest",
					)
					.await?
					.convert_to_solana_mint_token_req();

				// let args = Encode!(&unconfirmed_ticket.clone().ticket_id.clone())?;
				// let ret = agent
				// 		.query(&canister_id, "mint_token_req")
				// 		.with_arg(args)
				// 		.call()
				// 		.await?;
				// let mint_token_req: MintTokenRequest = Decode!(&ret, MintTokenRequest)?;

				info!(
					"Solana Mint Token Status: {:?} ",
					mint_token_req.clone().status
				);

				if let TxStatus::Finalized = mint_token_req.status {
					Mutation::update_ticket(
						db,
						unconfirmed_ticket.clone(),
						Some(TicketStatus::Finalized),
						Some(mint_token_req.signature),
						None,
						None,
						None,
						None,
					)
					.await?;
				}

				// match mint_token_req.status {
				// 	TxStatus::Finalized => {
				// 		Mutation::update_ticket(
				// 			db,
				// 			unconfirmed_ticket.clone(),
				// 			Some(TicketStatus::Finalized),
				// 			Some(mint_token_req.signature),
				// 			None,
				// 			None,
				// 			None,
				// 			None,
				// 		)
				// 		.await?;
				// 	}
				// 	TxStatus::Unknown => {
				// 		info!("{:?} is Unknown in Solana", unconfirmed_ticket.clone())
				// 	}
				// 	TxStatus::TxFailed { e } => {
				// 		info!("Solana error: {:?}  ", e)
				// 	}
				// }
			}

			Ok(())
		},
	)
	.await
}
