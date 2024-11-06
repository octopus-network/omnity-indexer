use crate::entity::ticket;
use crate::service::{Mutation, Query};
use crate::{token_ledger_id_on_chain, with_omnity_canister, Arg};
use ic_agent::{export::Principal, Agent};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const ICP_CUSTOM_CHAIN_ID: &str = "sICP";

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ICPCustomRelaseTokenStatus {
	Finalized { tx_hash: String },
	Unknown,
}

async fn process_ticket_status_from_sicp(
	db: &DbConn,
	unconfirmed_ticket: ticket::Model,
	agent: &Agent,
	canister_id: Principal,
) -> Result<(), Box<dyn Error>> {
	let release_icp_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
		.query_method(
			agent.clone(),
			canister_id,
			"mint_token_status",
			"Syncing mint token status from icp custom: ",
			"Release icp custom token status result: ",
			None,
			None,
			"ICPCustomRelaseTokenStatus",
		)
		.await?
		.convert_to_release_icp_token_status();

	if let ICPCustomRelaseTokenStatus::Finalized { tx_hash } = release_icp_token_status {
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
			"icp custom ticket id({:?}) finally status:{:?} and its hash is {:?} ",
			ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
		);
	}
	Ok(())
}

// sync tickets status that transfered from routes to icp custom
pub async fn sync_ticket_status_from_sicp(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			info!("Syncing release token status from icp custom ... ");

			if let (Ok(unconfirmed_tickets), _) = (
				Query::get_unconfirmed_tickets(db, ICP_CUSTOM_CHAIN_ID.to_owned()).await,
				Query::get_unconfirmed_deleted_tickets(db, ICP_CUSTOM_CHAIN_ID.to_owned()).await,
			) {
				for unconfirmed_ticket in unconfirmed_tickets {
					let _ = process_ticket_status_from_sicp(
						db,
						unconfirmed_ticket,
						&agent,
						canister_id,
					);
				}
			} else if let (_, Ok(unconfirmed_tickets)) = (
				Query::get_unconfirmed_tickets(db, ICP_CUSTOM_CHAIN_ID.to_owned()).await,
				Query::get_unconfirmed_deleted_tickets(db, ICP_CUSTOM_CHAIN_ID.to_owned()).await,
			) {
				for unconfirmed_ticket in unconfirmed_tickets {
					let _unconfirmed_ticket =
						ticket::Model::from_deleted_ticket(unconfirmed_ticket);
					let _ = process_ticket_status_from_sicp(
						db,
						_unconfirmed_ticket,
						&agent,
						canister_id,
					);
				}
			}
			Ok(())
		},
	)
	.await
}

pub async fn sync_all_icrc_token_canister_id_from_sicp(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			let token_canisters = Arg::V(Vec::<u8>::new())
				.query_method(
					agent.clone(),
					canister_id,
					"get_token_list",
					"Syncing token canister id from sicp ...",
					"Token canister id from sicp result: ",
					None,
					None,
					"Vec<Token>",
				)
				.await?
				.convert_to_vec_token();
			for token in token_canisters {
				if let Some(canister) = token.metadata.get("ledger_id") {
					let token_canister_id_on_chain_model = token_ledger_id_on_chain::Model::new(
						ICP_CUSTOM_CHAIN_ID.to_string(),
						token.token_id,
						canister.to_owned(),
					);

					let token_canister_id_on_chain = Mutation::save_all_token_ledger_id_on_chain(
						db,
						token_canister_id_on_chain_model,
					)
					.await?;

					info!(
						"Token {:?} in Chain id({:?})' Canister id is {:?}",
						token_canister_id_on_chain.token_id,
						token_canister_id_on_chain.chain_id,
						token_canister_id_on_chain.contract_id
					);
				}
			}

			Ok(())
		},
	)
	.await
}
