#[cfg(test)]
mod tests {
    use std::sync::Once;
    use super::*;
    use crate::entity::sea_orm_active_enums;
    use crate::{get_timestamp, random_txid, types, Database};
    use dotenvy::dotenv;
    const RUNE_ID: &str = "40000:846";
    const TOKEN_ID: &str = "Bitcoin-runes-HOPE•YOU•GET•RICH";
    static INIT: Once = Once::new();
    pub fn init_logger() {
        std::env::set_var("RUST_LOG", "info");
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
        });
    }

    #[ignore]
    #[test]
    fn test_generate_and_sync_ticket() {
        dotenv().ok();
        init_logger();
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
        let ticket = runtime.block_on(async {
            let txid = random_txid();
            let db = Database::new(db_url).await;
            let args = GenerateTicketArgs {
                target_chain_id: "eICP".to_owned(),
                receiver: String::from("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv"),
                rune_id: RUNE_ID.into(),
                amount: 1000,
                txid: txid.to_string(),
            };
            let _ = gen_bitcoin_ticket(args).await;

            let _ = sync_pending_tickets_from_bitcoin(&db.get_connection()).await;
            Query::get_ticket_by_id(&db.get_connection(), txid.to_string())
                .await
                .unwrap()
        });
        info!("synced pending ticket from bitcoin custom: {:#?}", ticket);
    }
    #[ignore]
    #[test]
    fn test_finalized_and_sync_ticket() {
        dotenv().ok();
        init_logger();
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
        runtime.block_on(async {
            let txid = random_txid();
            let db = Database::new(db_url).await;
            let args = GenerateTicketArgs {
                target_chain_id: "eICP".to_owned(),
                receiver: String::from("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv"),
                rune_id: RUNE_ID.into(),
                amount: 1000,
                txid: txid.to_string(),
            };
            let _ = gen_bitcoin_ticket(args).await;
            let _ = sync_pending_tickets_from_bitcoin(&db.get_connection()).await;
            let ticket = Query::get_ticket_by_id(&db.get_connection(), txid.to_string())
                .await
                .unwrap()
                .unwrap();
            info!("synced pending ticket from bitcoin: {:#?}", ticket);
            //mock remove ticket from pending tickets
            let _ = mock_finalized_ticket(txid.to_string()).await;
            let _ = sync_pending_tickets_from_bitcoin(&db.get_connection()).await;
            let updated_ticket = Mutation::update_ticket_status(
                &db.get_connection(),
                ticket.to_owned(),
                sea_orm_active_enums::TicketStatus::Finalized,
            )
            .await;
            info!("updated ticket status:{:#?}", updated_ticket);
        });
    }
    #[ignore]
    #[test]
    fn test_finalized_token_and_sync_ticket() {
        dotenv().ok();
        init_logger();
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
        runtime.block_on(async {
            let db = Database::new(db_url).await;
            let txid = random_txid();
            let ticket = types::Ticket {
                ticket_id: txid.to_string(),
                ticket_seq: None,
                ticket_type: TicketType::Normal,
                ticket_time: get_timestamp(),
                src_chain: "eICP".to_owned(),
                dst_chain: CUSTOMS_CHAIN_ID.to_owned(),
                action: TxAction::Redeem,
                token: TOKEN_ID.into(),
                amount: 1000.to_string(),
                sender: Some(String::from(
                    "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv",
                )),
                receiver: "bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6".to_owned(),
                memo: None,
                status: TicketStatus::WaitingForConfirmByDest,
            };
            // save to db
            let _ = Mutation::save_ticket(&db.get_connection(), ticket.into()).await;
            let finalized_status = FinalizedStatus::Confirmed(txid.to_owned());
            // mock finalize the ticket by release token
            let _ = mock_finalized_release_token(txid.to_string(), finalized_status).await;

            let _ = sync_ticket_status_from_bitcoin(&db.get_connection()).await;
        });
    }
}