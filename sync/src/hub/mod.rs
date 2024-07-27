use crate::{
	service::{Mutation, Query},
	types::{self, Ticket},
	with_omnity_canister, Arg, ChainId, TokenId,
};
use log::info;
use sea_orm::DbConn;
use std::error::Error;

pub const FETCH_LIMIT: u64 = 50;
pub const CHAIN_SYNC_INTERVAL: u64 = 60;
pub const TOKEN_SYNC_INTERVAL: u64 = 60;
pub const TICKET_SYNC_INTERVAL: u64 = 5;
pub const TICKET_UPDATE_INTERVAL: u64 = 120;
pub const TOKEN_ON_CHAIN_SYNC_INTERVAL: u64 = 60;
// pub const PENDING_TICKET_SYNC_INTERVAL: u64 = 60;

// full synchronization for pending tickets
// pub async fn sync_pending_tickets(db: &DbConn) -> Result<(), Box<dyn Error>> {
// 	with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
// 		// let _ = Delete::remove_pending_ticket(db).await?;
// 		let pending_ticket_size = Arg::V(Vec::<u8>::new())
// 			.query_method(
// 				agent.clone(),
// 				canister_id,
// 				"get_pending_ticket_size",
// 				"Syncing pending tickets from hub ... ",
// 				"Total pending ticket size: ",
// 				None,
// 				None,
// 				"u64",
// 			)
// 			.await?
// 			.convert_to_u64();

// 		info!(
// 			"Need to fetch pending tickets size: {:?}",
// 			pending_ticket_size
// 		);

// 		let mut from_seq = 0u64;
// 		while from_seq < pending_ticket_size {
// 			let new_pending_tickets = Arg::U(from_seq)
// 				.query_method(
// 					agent.clone(),
// 					canister_id,
// 					"get_pending_tickets",
// 					"Next offset:",
// 					"Synced pending tickets : ",
// 					Some(FETCH_LIMIT),
// 					None,
// 					"Vec<(TicketId, OmnityTicket)>",
// 				)
// 				.await?
// 				.convert_to_vec_omnity_pending_ticket();

// 			if new_pending_tickets.is_empty() {
// 				break;
// 			}

// 			for (_ticket_id, pending_ticket) in new_pending_tickets.clone() {
// 				let pending_ticket_model = pending_ticket.into();
// 				Mutation::save_pending_ticket(db, pending_ticket_model).await?;
// 			}
// 			from_seq += new_pending_tickets.clone().len() as u64;
// 		}
// 		Ok(())
// 	})
// 	.await
// }

pub async fn update_sender(db: &DbConn) -> Result<(), Box<dyn Error>> {
	// Find the tickets with no sender
	let null_sender_tickets = Query::get_null_sender_tickets(db).await?;

	info!("There are {:?} senders are null", null_sender_tickets.len());

	loop {
		for ticket in null_sender_tickets.clone() {
			let client = reqwest::Client::new();
			let url = "https://mempool.space/api/tx/".to_string() + &ticket.clone().ticket_id;
			let response = client.get(url).send().await?;

			let body = response.text().await?;
			let mut a = match serde_json::from_str::<serde_json::Value>(&body) {
				Ok(v) => v,
				Err(_) => continue,
			};

			if let Some(vin) = a.get_mut("vin") {
				let sender = vin[0]["prevout"]["scriptpubkey_address"]
					.as_str()
					.unwrap()
					.to_string();

				// Insert the sender into the ticket meta
				let updated_ticket =
					Mutation::update_tikcet_sender(db, ticket.clone(), sender).await?;

				info!(
					"Ticket id({:?}) has changed its sender to {:?}",
					ticket.ticket_id, updated_ticket.sender
				);
			};
		}
		break;
	}
	Ok(())
}

// full synchronization for token on chain
pub async fn sync_tokens_on_chains(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
		let tokens_on_chains_size = Arg::V(Vec::<u8>::new())
			.query_method(
				agent.clone(),
				canister_id,
				"get_token_position_size",
				"Syncing tokens on chains ...",
				"Tokens on chain size: ",
				None,
				None,
				"u64",
			)
			.await?
			.convert_to_u64();

		let mut from_seq = 0u64;

		while from_seq < tokens_on_chains_size {
			let tokens_on_chains = Arg::CHA(None::<ChainId>)
				.query_method(
					agent.clone(),
					canister_id,
					"get_chain_tokens",
					"Syncing tokens on chains from offset ...",
					"Total tokens from chains from offset: ",
					Some(from_seq),
					Some(None::<TokenId>),
					"Vec<OmnityTokenOnChain>",
				)
				.await?
				.convert_to_vec_omnity_token_on_chain();

			if tokens_on_chains.is_empty() {
				break;
			}

			for _token_on_chain in tokens_on_chains.iter() {
				Mutation::save_token_on_chain(db, _token_on_chain.clone().into()).await?;
			}
			from_seq += tokens_on_chains.len() as u64;
		}
		Ok(())
	})
	.await
}

// full synchronization for chains
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
					None,
					"Vec<ChainMeta>",
				)
				.await?
				.convert_to_vec_chain_meta();

			if chains.is_empty() {
				break;
			}

			for chain in chains.iter() {
				Mutation::save_chain(db, chain.clone().into()).await?;
			}
			from_seq += chains.len() as u64;
		}
		Ok(())
	})
	.await
}

// full synchronization for tokens
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
					None,
					"Vec<TokenMeta>",
				)
				.await?
				.convert_to_vec_token_meta();

			if tokens.is_empty() {
				break;
			}

			for token in tokens.iter() {
				Mutation::save_token(db, token.clone().into()).await?;
			}
			offset += tokens.len() as u64;
		}
		Ok(())
	})
	.await
}

// increment synchronization for ledger tickets and pending tickets
pub async fn sync_tickets(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
		// Ledger tickets
		let ticket_size = Arg::V(Vec::<u8>::new())
			.query_method(
				agent.clone(),
				canister_id,
				"sync_ticket_size",
				"Syncing tickets from hub ... ",
				"Total ticket size: ",
				None,
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
					None,
					"Vec<(u64, OmnityTicket)>",
				)
				.await?
				.convert_to_vec_omnity_ticket();

			if new_tickets.len() < limit as usize {
				break;
			}

			for (seq, ticket) in new_tickets.iter() {
				let ticket_modle = Ticket::from_omnity_ticket(*seq, ticket.clone()).into();
				Mutation::save_ticket(db, ticket_modle).await?;
			}
		}

		// Pending tickets
		let pending_ticket_size = Arg::V(Vec::<u8>::new())
			.query_method(
				agent.clone(),
				canister_id,
				"get_pending_ticket_size",
				"Syncing pending tickets from hub ... ",
				"Total pending ticket size: ",
				None,
				None,
				"u64",
			)
			.await?
			.convert_to_u64();
		info!(
			"Need to fetch pending tickets size: {:?}",
			pending_ticket_size
		);

		let mut from_seq = 0u64;
		while from_seq < pending_ticket_size {
			let new_pending_tickets = Arg::U(from_seq)
				.query_method(
					agent.clone(),
					canister_id,
					"get_pending_tickets",
					"Next offset:",
					"Synced pending tickets : ",
					Some(FETCH_LIMIT),
					None,
					"Vec<(TicketId, OmnityTicket)>",
				)
				.await?
				.convert_to_vec_omnity_pending_ticket();

			if new_pending_tickets.is_empty() {
				break;
			}

			for (_ticket_id, pending_ticket) in new_pending_tickets.clone() {
				// let pending_ticket_model = pending_ticket.into();
				// Mutation::save_pending_ticket(db, pending_ticket_model).await?;
				let ticket_model =
					Ticket::from_omnity_pending_ticket(_ticket_id, pending_ticket).into();
				Mutation::save_ticket(db, ticket_model).await?;
			}
			from_seq += new_pending_tickets.clone().len() as u64;
		}

		Ok(())
	})
	.await
}

// mocking
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
				None,
				"()",
			)
			.await?;

		Ok(())
	})
	.await
}
