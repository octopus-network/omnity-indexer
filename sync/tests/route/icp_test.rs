#[cfg(test)]
mod tests {
	use dotenvy::dotenv;
	use omnity_indexer_sync::{
		get_timestamp,
		icp::{mock_finalized_mint_token, sync_ticket_status_from_icp_route, ROUTE_CHAIN_ID},
		random_txid, types, Database, Mutation,
	};
	use std::sync::Once;

	static INIT: Once = Once::new();

	pub fn init_logger() {
		std::env::set_var("RUST_LOG", "info");
		INIT.call_once(|| {
			let _ = env_logger::builder().is_test(true).try_init();
		});
	}

	// const RUNE_ID: &str = "40000:846";
	const TOKEN_ID: &str = "Bitcoin-runes-HOPE•YOU•GET•RICH";

	#[ignore]
	#[test]
	fn test_finalized_token_and_sync_ticket() {
		dotenv().ok();
		init_logger();
		let db_url = std::env::var("DATABASE_URL").unwrap();
		let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
		runtime.block_on(async {
			let db = Database::new(db_url).await;
			let ticket_id = random_txid().to_string();
			let ticket = types::Ticket {
				ticket_id: ticket_id.to_owned(),
				ticket_seq: None,
				ticket_type: types::TicketType::Normal,
				ticket_time: get_timestamp(),
				src_chain: "Bitcoin".to_owned(),
				dst_chain: ROUTE_CHAIN_ID.to_owned(),
				action: types::TxAction::Transfer,
				token: TOKEN_ID.into(),
				amount: 1000.to_string(),
				sender: None,
				receiver: String::from("bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6"),
				memo: None,
				status: types::TicketStatus::WaitingForConfirmByDest,
			};
			// save to db
			let _ = Mutation::save_ticket(&db.get_connection(), ticket.into()).await;

			let _ = mock_finalized_mint_token(ticket_id.to_owned(), 1000).await;

			let _ = sync_ticket_status_from_icp_route(&db.get_connection()).await;
		});
	}
}
