use crate::entity::{sea_orm_active_enums::TicketStatus, ticket};
use crate::service::{Mutation, Query};
use crate::{token_ledger_id_on_chain, with_omnity_canister, Arg};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const ROUTE_CHAIN_ID: &str = "eICP";

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IcpMintTokenStatus {
	Finalized { block_index: u64 },
	Unknown,
}

pub async fn sync_all_icp_token_ledger_id_on_chain(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_ROUTES_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			info!("icp token ledger id on chain在工作 ... ");
			for token in Query::get_all_tokens(db).await? {
				let token_ledger = Arg::TokId(token.clone().token_id)
					.query_method(
						agent.clone(),
						canister_id,
						"get_token_ledger",
						None,
						None,
						"Option<Principal>",
					)
					.await?
					.convert_to_canister_id();
				if let Some(ledger_id) = token_ledger {
					let mut token_ledger_id = serde_json::to_string(&ledger_id).unwrap();
					token_ledger_id.replace_range(0..1, "");
					token_ledger_id.replace_range((token_ledger_id.len() - 1).., "");

					let token_ledger_id_on_chain_model = token_ledger_id_on_chain::Model::new(
						ROUTE_CHAIN_ID.to_owned(),
						token.clone().token_id,
						token_ledger_id,
					);
					let _token_ledger_id_on_chain = Mutation::save_all_token_ledger_id_on_chain(
						db,
						token_ledger_id_on_chain_model,
					)
					.await?;
				}
			}

			Ok(())
		},
	)
	.await
}

pub async fn sync_ticket_status_from_icp_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	if let (Ok(unconfirmed_tickets), Ok(deleted_unconfirmed_tickets)) = (
		Query::get_unconfirmed_tickets(db, ROUTE_CHAIN_ID.to_owned()).await,
		Query::get_unconfirmed_deleted_tickets(db, ROUTE_CHAIN_ID.to_owned()).await,
	) {
		for unconfirmed_ticket in unconfirmed_tickets {
			ticket_status_from_icp_route(db, unconfirmed_ticket).await?;
		}
		for deleted_unconfirmed_ticket in deleted_unconfirmed_tickets {
			let formated_deleted_unconfirmed_ticket =
				ticket::Model::from_deleted_ticket(deleted_unconfirmed_ticket);
			ticket_status_from_icp_route(db, formated_deleted_unconfirmed_ticket.clone()).await?;
		}
	}
	Ok(())
}

async fn ticket_status_from_icp_route(
	db: &DbConn,
	ticket: ticket::Model,
) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_ROUTES_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			info!("icp route状态更新在工作 ... ");
			let mint_token_status = Arg::TI(ticket.ticket_id.clone())
				.query_method(
					agent.clone(),
					canister_id,
					"mint_token_status",
					None,
					None,
					"IcpMintTokenStatus",
				)
				.await?
				.convert_to_icp_mint_token_status();

			if let IcpMintTokenStatus::Finalized { block_index } = mint_token_status {
				if let Some(rep) = Query::get_token_ledger_id_on_chain_by_id(
					db,
					ROUTE_CHAIN_ID.to_owned(),
					ticket.clone().token,
				)
				.await?
				{
					let tx_hash = rep.contract_id + "_" + &block_index.to_string();

					// update ticket status to finalized
					if let Ok(ticket_model) = Mutation::update_ticket(
						db,
						ticket.clone(),
						Some(TicketStatus::Finalized),
						Some(Some(tx_hash.clone())),
						None,
						None,
						None,
						None,
					)
					.await
					{
						info!(
							"Ticket id({:?}) status:{:?} and finalized on block {:?}",
							ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
						);
					} else if let Ok(d_ticket_model) =
						Mutation::update_deleted_ticket_statu_and_tx_hash(
							db,
							ticket.into(),
							Some(tx_hash),
							TicketStatus::Finalized,
						)
						.await
					{
						info!(
							"Deleted ticket id({:?}) status:{:?} and finalized on block {:?}",
							d_ticket_model.ticket_id, d_ticket_model.status, d_ticket_model.tx_hash
						);
					}
				}
			}
			Ok(())
		},
	)
	.await
}
