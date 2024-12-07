use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::routes::MintTokenStatus;
use crate::service::{Mutation, Query};
use crate::{token_ledger_id_on_chain, with_omnity_canister, Arg, ChainId};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct OsmoRoute {
	pub canister: &'static str,
	pub chain: ChainId,
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
			info!("Syncing release token status from osmosis ... ");

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
						"MintTokenStatus",
					)
					.await?
					.convert_to_mint_token_status();

				if let MintTokenStatus::Finalized { tx_hash } = mint_osmosis_token_status {
					let ticket_model = Mutation::update_ticket(
						db,
						unconfirmed_ticket.clone(),
						Some(TicketStatus::Finalized),
						Some(Some(tx_hash)),
						None,
						None,
						None,
						None,
					)
					.await?;

					info!(
						"osmosis route ticket id({:?}) status:{:?} and its hash is {:?} ",
						ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
					);
				}
			}

			Ok(())
		})
		.await?
	}
	Ok(())
}

pub async fn sync_all_cosmwasm_token_ledger_id_on_chain(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("OSMOSIS1_CHAIN_ID", |agent, canister_id| async move {
		let token_ledgers = Arg::V(Vec::<u8>::new())
			.query_method(
				agent.clone(),
				canister_id,
				"get_token_list",
				"Syncing token ledger id from osmosis route ...",
				"Token ledger id from osmosis route result: ",
				None,
				None,
				"Vec<CosmwasmTokenResp>",
			)
			.await?
			.convert_to_vec_cosmwasmtoken_resp();

		for cosmwasm_token_resp in token_ledgers {
			if let Some(token_denom) = &cosmwasm_token_resp.token_denom {
				let token_ledger_id_on_chain_model = token_ledger_id_on_chain::Model::new(
					"osmosis-1".to_owned(),
					cosmwasm_token_resp.token_id,
					token_denom.to_owned(),
				);
				// Save to the database
				let token_ledger_id_on_chain =
					Mutation::save_all_token_ledger_id_on_chain(db, token_ledger_id_on_chain_model)
						.await?;

				info!(
					"Token {:?} in Chain id({:?})' Contract id is {:?}",
					token_ledger_id_on_chain.token_id,
					token_ledger_id_on_chain.chain_id,
					token_ledger_id_on_chain.contract_id
				);
			}
		}

		Ok(())
	})
	.await?;

	Ok(())
}
