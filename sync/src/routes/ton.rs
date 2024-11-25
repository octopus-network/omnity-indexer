use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::routes::MintTokenStatus;
use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, Arg};
use log::info;
use sea_orm::DbConn;
use std::error::Error;
use std::str;

pub const TON_ROUTE_CHAIN_ID: &str = "Ton";

pub async fn sync_all_tickets_status_from_ton_route(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("TON_CANISTER_ID", |agent, canister_id| async move {
		info!("Syncing release token status from Ton ... ");
		let unconfirmed_tickets =
			Query::get_unconfirmed_tickets(db, TON_ROUTE_CHAIN_ID.to_owned()).await?;
		for unconfirmed_ticket in unconfirmed_tickets {
			let mint_ton_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
				.query_method(
					agent.clone(),
					canister_id,
					"mint_token_status",
					"Syncing mint token status from ton route ...",
					"Mint token status from ton route result: ",
					None,
					None,
					"MintTokenStatus",
				)
				.await?
				.convert_to_mint_token_status();

			if let MintTokenStatus::Finalized { tx_hash } = mint_ton_token_status {
				if let Ok(ticket_model) = Mutation::update_ticket(
					db,
					unconfirmed_ticket.clone(),
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
						"ton ticket id({:?}) status:{:?} and its hash is {:?} ",
						ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
					);
				}
			}
		}
		Ok(())
	})
	.await
}
