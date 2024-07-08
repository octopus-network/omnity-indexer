use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, Arg};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const BEVM_CHAIN_ID: &str = "bevm";
pub const BITLAYER_CHAIN_ID: &str = "bitlayer";
pub const XLAYER_CHAIN_ID: &str = "xlayer";
pub const BSQUARE_ID: &str = "b";
pub const MERLIN_ID: &str = "merlin";
pub const BOB_ID: &str = "bob";

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintEvmTokenStatus {
	Finalized { tx_hash: String },
	Unknown,
}

pub async fn sync_ticket_status_from_evm_route(
	db: &DbConn,
	canister_id: &str,
	chain_id: &str,
) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(canister_id, |agent, canister_id| async move {
		let unconfirmed_tickets = Query::get_unconfirmed_tickets(db, chain_id.to_owned()).await?;

		for unconfirmed_ticket in unconfirmed_tickets {
			let mint_evm_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
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
						"Ticket id({:?}) mint evm token status {:?}",
						unconfirmed_ticket.ticket_id,
						MintEvmTokenStatus::Unknown
					);
				}
				MintEvmTokenStatus::Finalized { tx_hash } => {
					info!(
						"Ticket id({:?}) finalized and its hash is {:?}",
						unconfirmed_ticket.ticket_id, tx_hash
					);

					// let tx_hash = Arg::TI(unconfirmed_ticket.ticket_id.clone())
					//     .query_method(
					//         agent.clone(),
					//         canister_id,
					//         "query_tx_hash",
					//         "Syncing the tx hash:",
					//         "Synced the tx hash : ",
					//         None,
					//         None,
					//         "Vec<(u64, OmnityTicket)>",
					//     )
					//     .await?
					//     .convert_to_tx_hash();

					let ticket_model = Mutation::update_ticket_status(
						db,
						unconfirmed_ticket,
						TicketStatus::Finalized,
					)
					.await?;
					info!(
						"Ticket id({:?}) status:{:?} ",
						ticket_model.ticket_id, ticket_model.status
					);
				}
			}
		}

		Ok(())
	})
	.await
}
