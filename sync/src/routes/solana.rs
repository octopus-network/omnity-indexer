use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, Arg};
use candid::CandidType;
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

pub async fn sync_ticket_status_from_solana_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_ROUTES_SOLANA_CANISTER_ID",
		|agent, canister_id| async move {
			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, SOLANA_ROUTE_CHAIN_ID.to_owned()).await?;

			for unconfirmed_ticket in unconfirmed_tickets {
				unconfirmed_ticket.clone().memo.unwrap().replace_range(0..2, "");
				let decoded_memo = hex::decode(unconfirmed_ticket.clone().memo.unwrap()).unwrap();
				let memo = str::from_utf8(&decoded_memo).unwrap().to_string();

				let _ = Mutation::update_ticket_memo(db, unconfirmed_ticket.clone(), memo).await?;

				let mint_token_status = Arg::TI(unconfirmed_ticket.clone().ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"mint_token_status",
						"Syncing mint token status from solana route ...",
						"Mint token status from solana route result: ",
						None,
						None,
						"TxStatus",
					)
					.await?
					.convert_to_mint_solana_token_status();

				if let TxStatus::Finalized = mint_token_status {
					let solana_hash = Arg::TI(unconfirmed_ticket.ticket_id.clone())
						.query_method(
							agent.clone(),
							canister_id,
							"mint_token_tx_hash",
							"",
							"",
							None,
							None,
							"Option<String>",
						)
						.await?
						.convert_to_mint_solana_token_status_hash();

					let _ = Mutation::update_ticket_status_n_txhash(
						db,
						unconfirmed_ticket.clone(),
						TicketStatus::Finalized,
						Some(solana_hash.unwrap()),
					)
					.await?;
				}
			}

			Ok(())
		},
	)
	.await
}
