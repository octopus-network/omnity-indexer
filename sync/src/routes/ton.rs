use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::routes::MintTokenStatus;
use crate::service::{Mutation, Query};
use crate::{token_ledger_id_on_chain, with_omnity_canister, Arg};
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
					" ",
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

pub async fn sync_all_ton_token_ledger_id_on_chain(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("TON_CANISTER_ID", |agent, canister_id| async move {
		let token_ledgers = Arg::V(Vec::<u8>::new())
			.query_method(
				agent.clone(),
				canister_id,
				"get_token_list",
				"Syncing token ledger id from ton routes ...",
				"  ",
				None,
				None,
				"Vec<TonTokenResp>",
			)
			.await?
			.convert_to_vec_ton_token_resp();
		for token_resp in token_ledgers {
			if let Some(ton_contract) = &token_resp.ton_contract {
				let token_ledger_id_on_chain_model = token_ledger_id_on_chain::Model::new(
					"Ton".to_owned(),
					token_resp.token_id,
					ton_contract.to_owned(),
				);
				// Save to the database
				let _token_ledger_id_on_chain =
					Mutation::save_all_token_ledger_id_on_chain(db, token_ledger_id_on_chain_model)
						.await?;
			}
		}

		Ok(())
	})
	.await
}
