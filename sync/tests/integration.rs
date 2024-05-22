mod omnity_indexer_sync_test {

    use std::str::FromStr;
    use std::sync::Once;

    use dotenvy::dotenv;
    use ic_btc_interface::Txid;
    use log::debug;
    use omnity_indexer_sync::service::Query;

    use omnity_indexer_sync::customs::bitcoin;
    use omnity_indexer_sync::customs::bitcoin::FinalizedStatus;
    use omnity_indexer_sync::customs::bitcoin::GenerateTicketArgs;
    use omnity_indexer_sync::hub;
    use omnity_indexer_sync::routes::icp;
    use omnity_indexer_sync::{get_timestamp, random_txid};

    use omnity_indexer_sync::types;
    use omnity_indexer_sync::types::TicketType;
    use omnity_indexer_sync::Database;
    const RUNE_ID: &str = "40000:846";
    const TOKEN_ID: &str = "Bitcoin-runes-HOPE•YOU•GET•RICH";
    static INIT: Once = Once::new();
    pub fn init_logger() {
        std::env::set_var("RUST_LOG", "debug");
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
        });
    }
    #[ignore]
    #[test]
    fn test_sync_tickets_custom2route() {
        dotenv().ok();
        init_logger();
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
        let ticket_model = runtime.block_on(async {
            let db = Database::new(db_url).await;
            //step1: mock to generate ticket from bitcoin custom
            let txid = random_txid();
            let args = GenerateTicketArgs {
                target_chain_id: "eICP".to_owned(),
                receiver: String::from("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv"),
                rune_id: RUNE_ID.into(),
                amount: 1000,
                txid: txid.to_string(),
            };

            let _ = bitcoin::gen_bitcoin_ticket(args).await;
            //step2: sync ticket from bitcoin custom
            let _ = bitcoin::sync_pending_tickets_from_bitcoin(&db.get_connection()).await;
            //mock remove ticket from pending tickets
            let _ = bitcoin::mock_finalized_ticket(txid.to_string()).await;
            //step3: mock,send ticket from custom to hub
            let ticket = types::Ticket {
                ticket_id: txid.to_string(),
                ticket_seq: None,
                ticket_type: TicketType::Normal,
                ticket_time: get_timestamp(),
                src_chain: "Bitcoin".to_owned(),
                dst_chain: "eICP".to_owned(),
                action: types::TxAction::Transfer,
                token: TOKEN_ID.into(),
                amount: 1000.to_string(),
                sender: None,
                receiver: "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_owned(),
                memo: None,
                status: types::TicketStatus::WaitingForConfirmByDest,
            };
            //step4: mock send ticket from custom to hub
            let _ = hub::send_tickets(ticket.to_owned()).await;
            //step5: sync ticket form hub
            let _ = hub::sync_tickets(&db.get_connection()).await;
            //step6: mock finalized mint token on route
            let _ = icp::mock_finalized_mint_token(ticket.ticket_id.to_owned(), 100).await;
            //step7: sync ticket status from route
            let _ = icp::sync_ticket_status_from_icp_route(&db.get_connection()).await;
            //step8: check ticket status
            Query::get_ticket_by_id(&db.get_connection(), ticket.ticket_id.to_owned())
                .await
                .unwrap()
        });
        debug!("finally ticket model:{:?}", ticket_model);
    }
    #[ignore]
    #[test]
    fn test_sync_tickets_route2custom() {
        dotenv().ok();
        init_logger();
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
        let ticket_model = runtime.block_on(async {
            let db = Database::new(db_url).await;
            //step1: mock,send ticket from route to hub
            let ticket_id = random_txid().to_string();
            let ticket = types::Ticket {
                ticket_id: ticket_id.to_owned(),
                ticket_seq: None,
                ticket_type: TicketType::Normal,
                ticket_time: get_timestamp(),
                src_chain: "eICP".to_owned(),
                dst_chain: "Bitcoin".to_owned(),
                action: types::TxAction::Redeem,
                token: TOKEN_ID.into(),
                amount: 1000.to_string(),
                sender: Some(String::from(
                    "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv",
                )),
                receiver: "bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6".to_owned(),
                memo: None,

                status: types::TicketStatus::WaitingForConfirmByDest,
            };
            //step4: mock send ticket from route to hub
            let _ = hub::send_tickets(ticket.to_owned()).await;
            //step5: sync ticket form hub
            let _ = hub::sync_tickets(&db.get_connection()).await;
            //step6: mock finalized release token on bitcoin custom
            let txid = Txid::from_str(&ticket.ticket_id.to_owned()).unwrap();
            let status = FinalizedStatus::Confirmed(txid);
            let _ =
                bitcoin::mock_finalized_release_token(ticket.ticket_id.to_owned(), status).await;
            //step7: sync ticket status from route
            let _ = bitcoin::sync_ticket_status_from_bitcoin(&db.get_connection()).await;
            //step8: check ticket status
            Query::get_ticket_by_id(&db.get_connection(), ticket.ticket_id.to_owned())
                .await
                .unwrap()
        });
        debug!("finally ticket model:{:?}", ticket_model);
    }
}
