use crate::{
	service::{Mutation, Query},
	types::{self, Ticket},
	with_omnity_canister, Arg,
};
use log::info;
use sea_orm::DbConn;
use std::error::Error;

const FETCH_LIMIT: u64 = 50;
pub const CHAIN_SYNC_INTERVAL: u64 = 5;
pub const TOKEN_SYNC_INTERVAL: u64 = 5;
pub const TICKET_SYNC_INTERVAL: u64 = 3;

//full synchronization for chains
pub async fn sync_chains(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
		let chain_size = Arg::query_method(
			Arg::V(Vec::<u8>::new()),
			agent.clone(),
			canister_id,
			"get_chain_size",
			"Syncing chains ...",
			"Chain size: ",
			None,
			"u64",
		)
		.await?
		.convert_to_u64();

		let mut from_seq = 0u64;
		while from_seq < chain_size {
			let chains = Arg::U(from_seq)
				.query_method(
					agent.clone(),
					canister_id,
					"get_chain_metas",
					"Syncing chains metadata ...",
					"Sync chains from offset: ",
					Some(FETCH_LIMIT),
					"Vec<ChainMeta>",
				)
				.await?
				.convert_to_vec_chain_meta();

			for chain in chains.iter() {
				Mutation::save_chain(db, chain.clone().into()).await?;
			}
			from_seq += chains.len() as u64;
			if chains.is_empty() {
				break;
			}
		}
		Ok(())
	})
	.await
}

//full synchronization for tokens
pub async fn sync_tokens(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
		let token_size = Arg::V(Vec::<u8>::new())
			.query_method(
				agent.clone(),
				canister_id,
				"get_token_size",
				"Syncing tokens ... ",
				"Total token size: ",
				None,
				"u64",
			)
			.await?
			.convert_to_u64();

		let mut offset = 0u64;
		while offset < token_size {
			let tokens = Arg::U(offset)
				.query_method(
					agent.clone(),
					canister_id,
					"get_token_metas",
					"Syncing tokens metadata ...",
					"Total tokens from offset: ",
					Some(FETCH_LIMIT),
					"Vec<TokenMeta>",
				)
				.await?
				.convert_to_vec_token_meta();

			for token in tokens.iter() {
				Mutation::save_token(db, token.clone().into()).await?;
			}
			offset += tokens.len() as u64;
			if tokens.is_empty() {
				break;
			}
		}

		Ok(())
	})
	.await
}

pub async fn send_tickets(ticket: types::Ticket) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
		let _ = Arg::T(ticket)
			.query_method(
				agent.clone(),
				canister_id,
				"send_ticket",
				"Send tickets to hub...",
				"Send ticket result: ",
				None,
				"()",
			)
			.await?;

		Ok(())
	})
	.await
}

//increment synchronization for tickets
pub async fn sync_tickets(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
		let ticket_size = Arg::V(Vec::<u8>::new())
			.query_method(
				agent.clone(),
				canister_id,
				"sync_ticket_size",
				"Syncing tickets from hub ... ",
				"Total ticket size: ",
				None,
				"u64",
			)
			.await?
			.convert_to_u64();

		//get latest ticket seq from  postgresql database
		let latest_ticket_seq = Query::get_latest_ticket(db).await?.map(|t| {
			info!("Latest ticket : {:?}", t);
			t.ticket_seq
		});
		let offset = match latest_ticket_seq {
			Some(t) => {
				info!("Latest ticket seq: {:?}", t);
				// the latest ticket seq may be Some or may be None
				t.map_or(0u64, |t| (t + 1) as u64)
			}
			None => {
				info!("No tickets found");
				0u64
			}
		};

		let tickets_to_fetch = ticket_size.saturating_sub(offset);
		info!("Need to fetch tickets size: {:?}", tickets_to_fetch);

		let mut limit = FETCH_LIMIT;
		for next_offset in (offset..ticket_size).step_by(limit as usize) {
			limit = std::cmp::min(limit, ticket_size - next_offset);
			let new_tickets = Arg::U(next_offset)
				.query_method(
					agent.clone(),
					canister_id,
					"sync_tickets",
					"Next_offset:",
					"Synced tickets : ",
					Some(limit),
					"Vec<(u64, OmnityTicket)>",
				)
				.await?
				.convert_to_vec_omnity_ticket();

			for (seq, ticket) in new_tickets.iter() {
				let ticket_modle = Ticket::from_omnity_ticket(*seq, ticket.clone()).into();
				Mutation::save_ticket(db, ticket_modle).await?;
			}
			if new_tickets.len() < limit as usize {
				break;
			}
		}
		Ok(())
	})
	.await
}
