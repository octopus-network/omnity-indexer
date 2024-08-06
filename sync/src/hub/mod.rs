use crate::{
	routes::icp::{MintTokenStatus, ROUTE_CHAIN_ID},
	service::{Mutation, Query},
	ticket, with_omnity_canister, Arg, ChainId, TokenId,
};
use log::info;
use sea_orm::DbConn;
use std::error::Error;

pub const FETCH_LIMIT: u64 = 50;
pub const CHAIN_SYNC_INTERVAL: u64 = 300;
pub const TOKEN_SYNC_INTERVAL: u64 = 300;
pub const TICKET_SYNC_INTERVAL: u64 = 5;
pub const TOKEN_ON_CHAIN_SYNC_INTERVAL: u64 = 120;

pub async fn update_from_deleted_mint_ticket(db: &DbConn) -> Result<(), Box<dyn Error>> {
	// Fetch the deleted mint tickets
	let deleted_tickets = Query::get_all_deleted_mint_ticket(db).await?;

	for ticket in deleted_tickets {
		// ICP side // todo: EVM side(use match)
		if ticket.dst_chain == ROUTE_CHAIN_ID {
			let _ = with_omnity_canister(
				"OMNITY_ROUTES_ICP_CANISTER_ID",
				|agent, canister_id| async move {
					let unconfirmed_tickets =
						Query::get_unconfirmed_deleted_mint_tickets(db, ROUTE_CHAIN_ID.to_owned())
							.await?;

					for unconfirmed_ticket in unconfirmed_tickets {
						let mint_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
							.query_method(
								agent.clone(),
								canister_id,
								"mint_token_status",
								"Syncing mint token status in deleted mint ticket from icp route ...",
								"Mint token status in deleted mint ticket from icp route result: ",
								None,
								None,
								"MintTokenStatus",
							)
							.await?
							.convert_to_mint_token_status();

						if let MintTokenStatus::Finalized { block_index } = mint_token_status {
							let tx_hash = match Query::get_token_ledger_id_on_chain_by_id(
								db,
								ROUTE_CHAIN_ID.to_owned(),
								unconfirmed_ticket.clone().token,
							)
							.await?
							{
								Some(rep) => rep.contract_id + "-" + &block_index.to_string(),
								None => block_index.to_string(),
							};

							let _ = Mutation::update_deleted_mint_ticket_status_n_txhash(
								db,
								unconfirmed_ticket.clone(),
								crate::entity::sea_orm_active_enums::TicketStatus::Finalized,
								Some(tx_hash),
							)
							.await?;
						}
					}
					Ok(())
				},
			)
			.await?;

			if let Some(tx_hash) = ticket.tx_hash {
				let updated_mint_tickets = Query::get_updated_mint_tickets(db).await?;
				for mint_ticket in updated_mint_tickets {
					// fetch the tx_hash from the mint ticket and put it in intermediate_tx_hash
					let intermediate_tx_hash = mint_ticket.clone().tx_hash;
					let _ = Mutation::update_ticket_intermediate_tx_hash(
						db,
						mint_ticket.clone(),
						intermediate_tx_hash,
					)
					.await?;
					// put the hash to mint ticket tx_hash
					let _ = Mutation::update_ticket_tx_hash(db, mint_ticket, Some(tx_hash.clone()))
						.await?;
				}
			}
		}
		// Update sender/seq and remove intermediate_tx_hash field only if they are needed
	}
	Ok(())
}

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
				let ticket_modle = ticket::Model::from_omnity_ticket(*seq, ticket.clone()).into();
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
				let ticket_model = ticket::Model::from_omnity_pending_ticket(pending_ticket).into();
				Mutation::save_ticket(db, ticket_model).await?;
			}
			from_seq += new_pending_tickets.clone().len() as u64;
		}

		Ok(())
	})
	.await
}
