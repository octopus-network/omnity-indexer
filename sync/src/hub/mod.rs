use crate::{
	pending_ticket,
	service::{Mutation, Query},
	ticket, token_volume, with_omnity_canister, Arg, ChainId, TokenId,
};
use log::info;
use sea_orm::DbConn;
use std::{error::Error, str};

pub const FETCH_LIMIT: u64 = 50;
pub const CHAIN_SYNC_INTERVAL: u64 = 1800;
pub const TOKEN_SYNC_INTERVAL: u64 = 1800;
pub const TICKET_SYNC_INTERVAL: u64 = 8;
pub const TOKEN_ON_CHAIN_SYNC_INTERVAL: u64 = 600;
pub const TOKEN_VOLUME_SYNC_INTERVAL: u64 = 60;

pub async fn update_volume(db: &DbConn) -> Result<(), Box<dyn Error>> {
	info!("Syncing with token volumes...");

	for token in Query::get_all_tokens(db).await? {
		let token_tickets = Query::get_token_tickets(db, token.clone().token_id).await?;
		let mut total: u128 = 0;
		let total_len = token_tickets.len();
		let mut count = 0;
		for t in token_tickets {
			total += t.amount.parse::<u128>().unwrap_or(0);
			count += 1;
			if count == total_len {
				let _token_volume =
					token_volume::Model::new(token.clone().token_id, total_len, total);
				Mutation::save_token_volume(db, _token_volume.clone().into()).await?;
			}
		}
	}
	Ok(())
}

pub async fn update_sender(db: &DbConn) -> Result<(), Box<dyn Error>> {
	// Find the tickets with no sender
	let null_sender_tickets = Query::get_null_sender_tickets(db).await?;
	info!("There are {:?} senders are null", null_sender_tickets.len());
	if null_sender_tickets.len() > 0 {
		loop {
			for ticket in null_sender_tickets.clone() {
				let client = reqwest::Client::builder()
					.timeout(std::time::Duration::new(8, 0))
					.build()?;
				let url = "https://mempool.space/api/tx/".to_string() + &ticket.clone().ticket_id;
				match client.get(url).header("Origin", "*").send().await {
					Ok(response) => {
						match response.text().await {
							Ok(body) => {
								let mut a = match serde_json::from_str::<serde_json::Value>(&body) {
									Ok(v) => v,
									Err(_) => {
										let updated_ticket = Mutation::update_ticket(
											db,
											ticket.clone(),
											None,
											None,
											None,
											Some(Some("unavailable sender".to_string())),
											None,
											None,
										)
										.await?;
										info!(
											"Ticket id({:?}) has changed its sender to {:?}",
											ticket.ticket_id, updated_ticket.sender
										);
										continue;
									}
								};
	
								if let Some(vin) = a.get_mut("vin") {
									if let Some(sender) =
										vin[0]["prevout"]["scriptpubkey_address"].as_str()
									{
										let _sender = sender.to_string();
										// Insert the sender into the ticket meta
										let updated_ticket = Mutation::update_ticket(
											db,
											ticket.clone(),
											None,
											None,
											None,
											Some(Some(_sender)),
											None,
											None,
										)
										.await?;
	
										info!(
											"Ticket id({:?}) has changed its sender to {:?}",
											ticket.ticket_id, updated_ticket.sender
										);
									}
								};
							}
							Err(e) => {
								info!("Mempool error: {:?}", e);
								continue;
							}
						}
					}
					Err(e) => {
						info!("Mempool errors: {:?}", e);
						continue;
					}
				}
			}
			break;
		}
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

// increment synchronization for ledger tickets and full synchronization for pending tickets
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
					"Syncing tickets from hub ...",
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
				let mut updated_memo = None;
				if let Some(memo) = ticket.clone().memo {
					if memo.len() > 0 {
						if let Ok(new_ticket_memo) = str::from_utf8(&memo) {
							updated_memo = Some(new_ticket_memo.to_string());
						}
					}
				}

				let ticket_modle =
					ticket::Model::from_omnity_ticket(*seq, ticket.clone(), updated_memo).into();
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

		let latest_ticket_index = Query::get_latest_pending_ticket(db).await?.map(|t| {
			info!("last pending ticket : {:?}", t);
			t.ticket_index
		});

		let pending_offset = match latest_ticket_index {
			Some(t) => {
				info!("Latest pending ticket: {:?}", t);
				t as u64
			}
			None => {
				info!("No tickets found");
				0u64
			}
		};

		let mut pending_limit = FETCH_LIMIT;
		for pending_next_offset in
			(pending_offset..pending_ticket_size).step_by(pending_limit as usize)
		{
			pending_limit = std::cmp::min(pending_limit, pending_ticket_size - pending_next_offset);
			let new_pending_tickets = Arg::U(pending_next_offset)
				.query_method(
					agent.clone(),
					canister_id,
					"get_pending_tickets",
					"Synced pending tickets now :",
					"Synced pending tickets result: ",
					Some(pending_limit),
					None,
					"Vec<(TicketId, OmnityTicket)>",
				)
				.await?
				.convert_to_vec_omnity_pending_ticket();

			if new_pending_tickets.len() < pending_limit as usize {
				break;
			}
			let last_index = pending_offset + pending_limit;

			for (_ticket_id, pending_ticket) in new_pending_tickets.iter() {
				let mut updated_memo = None;
				if let Some(memo) = pending_ticket.clone().memo {
					if memo.len() > 0 {
						if let Ok(new_ticket_memo) = str::from_utf8(&memo) {
							updated_memo = Some(new_ticket_memo.to_string());
						}
					}
				}

				let ticket_model = ticket::Model::from_omnity_pending_ticket(
					pending_ticket.clone().to_owned(),
					updated_memo,
				)
				.into();
				Mutation::save_ticket(db, ticket_model).await?;
			}

			// let pending_ticket_model = pending_ticket::Model::from_omnity_pending_ticket(
			// 	pending_ticket.clone().to_owned(),
			// 	updated_memo.clone(),
			// );
			// Mutation::save_pending_ticket(db, pending_ticket_model).await?;
			let pending_ticket_model = pending_ticket::Model::from_index(last_index as i32);
			Mutation::save_pending_ticket_index(db, pending_ticket_model).await?;
		}
		Ok(())
	})
	.await
}
