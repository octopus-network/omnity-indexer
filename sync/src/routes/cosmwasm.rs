use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, Arg, ChainId};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const OSMO_TEST_CHAIN_ID: &str = "osmo-test-5";
pub const OSMO_CHAIN_ID: &str = "osmosis-1";

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct OsmoRoute {
	pub canister: &'static str,
	pub chain: ChainId,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintCosmwasmTokenStatus {
	Finalized { tx_hash: String },
	Unknown,
}

pub async fn sync_all_tickets_status_from_cosmwasm_route(
	db: &DbConn,
) -> Result<(), Box<dyn Error>> {
	let osmosis_routes: Vec<OsmoRoute> = vec![
		OsmoRoute {
			canister: "OSMOSIS_TEST5_CHAIN_ID",
			chain: "osmo-test-5".to_owned(),
		},
		OsmoRoute {
			canister: "OSMOSIS1_CHAIN_ID",
			chain: "osmosis-1".to_owned(),
		},
	];

	for osmosis_route in osmosis_routes.iter() {
		with_omnity_canister(osmosis_route.canister, |agent, canister_id| async move {
			info!(
				"{:?} Syncing release token status from osmosis ... ",
				chrono::Utc::now()
			);

			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, osmosis_route.chain.clone()).await?;

			for unconfirmed_ticket in unconfirmed_tickets {
				let mint_osmosis_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"mint_token_status",
						"Syncing mint token status from osmosis route ...",
						"Mint token status from osmosis route result: ",
						None,
						None,
						"MintCosmwasmTokenStatus",
					)
					.await?
					.convert_to_mint_cosmwasm_token_status();

				match mint_osmosis_token_status {
					MintCosmwasmTokenStatus::Unknown => {
						info!(
							"Ticket id({:?}) from {:?} mint osmosis token status {:?}",
							unconfirmed_ticket.ticket_id,
							osmosis_route.chain.clone(),
							MintCosmwasmTokenStatus::Unknown
						);
					}
					MintCosmwasmTokenStatus::Finalized { tx_hash } => {
						let ticket_model = Mutation::update_ticket_status_n_txhash(
							db,
							unconfirmed_ticket.clone(),
							TicketStatus::Finalized,
							Some(tx_hash),
						)
						.await?;

						info!(
							"Ticket id({:?}) status:{:?} and its hash is {:?} ",
							ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
						);
					}
				}
			}

			Ok(())
		})
		.await?
	}
	Ok(())
}
