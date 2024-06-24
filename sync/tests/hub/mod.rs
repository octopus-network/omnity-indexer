mod hub_tests {
	use dotenvy::dotenv;
	use log::info;
	use omnity_indexer_sync::{
		get_timestamp, random_txid, send_tickets, service::Query, sync_chains, sync_tickets,
		sync_tokens, types, Database,
	};
	use std::sync::Once;

	static INIT: Once = Once::new();

	pub fn init_logger() {
		std::env::set_var("RUST_LOG", "info");
		INIT.call_once(|| {
			let _ = env_logger::builder().is_test(true).try_init();
		});
	}

	#[ignore]
	#[test]
	fn test_sync_chains() {
		dotenv().ok();
		init_logger();
		let db_url = std::env::var("TEST_DATABASE_URL").unwrap();
		let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
		let chains = runtime.block_on(async {
			let db = Database::new(db_url).await;
			let ret = sync_chains(&db.get_connection()).await;
			info!("sync_chains result{:?}", ret);
			Query::get_all_chains(&db.get_connection()).await.unwrap()
		});

		for chain in chains {
			let omnity_chain: types::ChainMeta = chain.into();
			info!("{:#?}", omnity_chain);
		}
	}
	#[ignore]
	#[test]
	fn test_sync_tokens() {
		dotenv().ok();
		init_logger();
		let db_url = std::env::var("TEST_DATABASE_URL").unwrap();
		let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
		let tokens = runtime.block_on(async {
			let db = Database::new(db_url).await;
			let ret = sync_tokens(&db.get_connection()).await;
			info!("sync_tokens result{:?}", ret);
			Query::get_all_tokens(&db.get_connection()).await.unwrap()
		});

		for token in tokens {
			let omnity_token: types::TokenMeta = token.into();
			info!("{:#?}", omnity_token);
		}
	}

	#[ignore]
	#[test]
	fn test_sync_tickets() {
		dotenv().ok();
		init_logger();
		let db_url = std::env::var("TEST_DATABASE_URL").unwrap();
		let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
		let tickets = runtime.block_on(async {
			let db = Database::new(db_url).await;
			let ret = sync_tickets(&db.get_connection()).await;
			info!("sync_tickets result{:?}", ret);
			Query::get_all_tickets(&db.get_connection()).await.unwrap()
		});

		for ticket in tickets {
			let omnity_ticket: types::Ticket = ticket.into();
			info!("{:#?}", omnity_ticket);
		}
	}

	#[ignore]
	#[test]
	fn test_send_tickets() {
		dotenv().ok();
		init_logger();
		let db_url = std::env::var("TEST_DATABASE_URL").unwrap();
		let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");

		let ticket_model = runtime.block_on(async {
			let txid = random_txid();
			let ticket = types::Ticket {
				ticket_id: txid.to_owned().to_string(),
				ticket_seq: None,
				ticket_type: types::TicketType::Normal,
				ticket_time: get_timestamp(),
				src_chain: "Bitcoin".to_owned(),
				dst_chain: "eICP".to_owned(),
				action: types::TxAction::Transfer,
				token: "Bitcoin-runes-HOPE•YOU•GET•RICH".to_owned(),
				amount: 1000.to_string(),
				sender: None,
				receiver: "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_owned(),
				memo: None,
				status: types::TicketStatus::WaitingForConfirmByDest,
			};
			let ret = send_tickets(ticket.to_owned()).await;
			info!("send_tickets result{:?}", ret);
			let db = Database::new(db_url).await;
			let ret = sync_tickets(&db.get_connection()).await;
			info!("sync_tickets result{:?}", ret);
			Query::get_ticket_by_id(&db.get_connection(), ticket.ticket_id.to_owned())
				.await
				.unwrap()
		});
		info!("send and synced latest ticket:{:#?}", ticket_model);
	}
}
