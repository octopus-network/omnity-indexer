use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, Arg, ChainId};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct EvmRoutes {
	pub canister: &'static str,
	pub chain: ChainId,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintEvmTokenStatus {
	Finalized { tx_hash: String },
	Unknown,
}

pub async fn sync_all_tickets_status_from_evm_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	let evm_routes = vec![
		EvmRoutes {
			canister: "BEVM_CHAIN_ID",
			chain: "bevm".to_owned(),
		},
		EvmRoutes {
			canister: "BITLAYER_CHAIN_ID",
			chain: "Bitlayer".to_owned(),
		},
		EvmRoutes {
			canister: "XLAYER_CHAIN_ID",
			chain: "X Layer".to_owned(),
		},
		EvmRoutes {
			canister: "BSQUARE_CHAIN_ID",
			chain: "B² Network".to_owned(),
		},
		EvmRoutes {
			canister: "MERLIN_CHAIN_ID",
			chain: "Merlin".to_owned(),
		},
		EvmRoutes {
			canister: "BOB_CHAIN_ID",
			chain: "Bob".to_owned(),
		},
	];

	for evm_route in evm_routes.iter() {
		let _ = sync_ticket_status_from_evm_route(db, evm_route.canister, evm_route.chain.clone())
			.await;
	}
	Ok(())
}

async fn sync_ticket_status_from_evm_route(
	db: &DbConn,
	canister_id: &str,
	chain_id: ChainId,
) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(canister_id, |agent, canister_id| async move {
		let unconfirmed_tickets = Query::get_unconfirmed_tickets(db, chain_id).await?;

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
					let ticket_tx_hash =
						Mutation::update_tikcet_tx_hash(db, unconfirmed_ticket.clone(), tx_hash)
							.await?;

					let ticket_model = Mutation::update_ticket_status(
						db,
						unconfirmed_ticket.clone(),
						TicketStatus::Finalized,
					)
					.await?;

					info!(
						"Ticket id({:?}) status:{:?} and its hash is {:?} ",
						ticket_model.ticket_id, ticket_model.status, ticket_tx_hash
					);
				}
			}
		}

		Ok(())
	})
	.await
}
