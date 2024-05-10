mod omnity_hub {

    use dotenvy::dotenv;
    use omnity_indexer_service::Query;
    use omnity_indexer_sync::hub::sync_chains;
    use omnity_indexer_sync::hub::sync_tickets;
    use omnity_indexer_sync::hub::sync_tokens;
    use omnity_indexer_sync::types;
    use omnity_indexer_sync::Database;

    #[ignore]
    #[test]
    fn test_sync_chains() {
        dotenv().ok();

        let db_url = std::env::var("DATABASE_URL").unwrap();
        let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
        let chains = runtime.block_on(async {
            let db = Database::new(db_url).await;
            sync_chains(&db.get_connection()).await;
            Query::get_all_chains(&db.get_connection()).await.unwrap()
        });

        for chain in chains {
            let omnity_chain: types::ChainMeta = chain.into();
            println!("{:#?}", omnity_chain);
        }
    }
    #[ignore]
    #[test]
    fn test_sync_tokens() {
        dotenv().ok();
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
        let tokens = runtime.block_on(async {
            let db = Database::new(db_url).await;
            sync_tokens(&db.get_connection()).await;
            Query::get_all_tokens(&db.get_connection()).await.unwrap()
        });

        for token in tokens {
            let omnity_token: types::TokenMeta = token.into();
            println!("{:#?}", omnity_token);
        }
    }
    #[ignore]
    #[test]
    fn test_sync_tickets() {
        dotenv().ok();
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let runtime = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");
        let tickets = runtime.block_on(async {
            let db = Database::new(db_url).await;
            sync_tickets(&db.get_connection()).await;
            Query::get_all_tickets(&db.get_connection()).await.unwrap()
        });

        for ticket in tickets {
            let omnity_ticket: types::Ticket = ticket.into();
            println!("{:#?}", omnity_ticket);
        }
    }
}
