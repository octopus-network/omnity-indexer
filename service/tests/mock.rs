mod prepare;

use ::entity::{chain_meta, notes, sea_orm_active_enums::*, ticket, token_meta};
use omnity_indexer_service::{Mutation, Query};
use prepare::{
    get_timestamp, prepare_mock_chains, prepare_mock_notes, prepare_mock_tickets,
    prepare_mock_tokens,
};
use sea_orm::*;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_note() {
    let db = &prepare_mock_notes();

    {
        let note = Query::find_note_by_id(db, 1).await.unwrap().unwrap();

        assert_eq!(note.id, 1);
    }

    {
        let note = Query::find_note_by_id(db, 5).await.unwrap().unwrap();

        assert_eq!(note.id, 5);
    }

    {
        let note = Mutation::create_note(
            db,
            notes::Model {
                id: 0,
                title: "Title D".to_owned(),
                text: "Text D".to_owned(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            note,
            notes::Model {
                id: 6,
                title: "Title D".to_owned(),
                text: "Text D".to_owned(),
            }
        );
    }

    {
        let note = Mutation::update_note_by_id(
            db,
            1,
            notes::Model {
                id: 1,
                title: "New Title A".to_owned(),
                text: "New Text A".to_owned(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            note,
            notes::Model {
                id: 1,
                title: "New Title A".to_owned(),
                text: "New Text A".to_owned(),
            }
        );
    }

    {
        let result = Mutation::delete_note(db, 5).await.unwrap();

        assert_eq!(result.rows_affected, 1);
    }

    {
        let result = Mutation::delete_all_notes(db).await.unwrap();

        assert_eq!(result.rows_affected, 5);
    }
}

#[tokio::test]
async fn test_chain_meta() {
    let db = &prepare_mock_chains();
    {
        let chains = Query::get_all_chains(db).await.unwrap();

        assert_eq!(chains.len(), 4);
        for chain in chains {
            println!("{:?}", chain);
        }
    }
    {
        let chain = Mutation::save_chain(
            db,
            chain_meta::Model {
                chain_id: "EVM-Arbitrum".to_string(),
                chain_type: ChainType::ExecutionChain,
                chain_state: ChainState::Active,
                canister_id: "bkyz2-fmaaa-aaasaa-qadaab-cai".to_string(),
                contract_address: Some("Arbitrum constract address".to_string()),
                counterparties: Some(
                    vec![
                        "Bitcoin".to_string(),
                        "Ethereum".to_string(),
                        "ICP".to_string(),
                    ]
                    .into(),
                ),
                fee_token: Some("Ethereum-ERC20-ARB".to_owned()),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            chain,
            chain_meta::Model {
                chain_id: "EVM-Arbitrum".to_string(),
                chain_type: ChainType::ExecutionChain,
                chain_state: ChainState::Active,
                canister_id: "bkyz2-fmaaa-aaasaa-qadaab-cai".to_string(),
                contract_address: Some("Arbitrum constract address".to_string()),
                counterparties: Some(
                    vec![
                        "Bitcoin".to_string(),
                        "Ethereum".to_string(),
                        "ICP".to_string(),
                    ]
                    .into()
                ),
                fee_token: Some("Ethereum-ERC20-ARB".to_owned()),
            },
        );
    }
}

#[tokio::test]
async fn test_token_meta() {
    let db = &prepare_mock_tokens();

    {
        let tokens = Query::get_all_tokens(db).await.unwrap();

        assert_eq!(tokens.len(), 4);
        for token in tokens {
            println!("{:?}", token);
        }
    }
    {
        let token = Mutation::save_token(
            db,
            token_meta::Model {
                token_id: "Ethereum-ERC20-ARB".to_string(),
                name: "ARB".to_owned(),
                symbol: "ARB".to_owned(),
                issue_chain: "Ethereum".to_string(),
                decimals: 18,
                icon: None,
                metadata: json!(HashMap::<String, String>::default()),
                dst_chains: vec![
                    "Bitcoin".to_string(),
                    "Ethereum".to_string(),
                    "ICP".to_string(),
                    "EVM-Optimistic".to_string(),
                    "EVM-Starknet".to_string(),
                ]
                .into(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            token,
            token_meta::Model {
                token_id: "Ethereum-ERC20-ARB".to_string(),
                name: "ARB".to_owned(),
                symbol: "ARB".to_owned(),
                issue_chain: "Ethereum".to_string(),
                decimals: 18,
                icon: None,
                metadata: json!(HashMap::<String, String>::default()),
                dst_chains: vec![
                    "Bitcoin".to_string(),
                    "Ethereum".to_string(),
                    "ICP".to_string(),
                    "EVM-Optimistic".to_string(),
                    "EVM-Starknet".to_string(),
                ]
                .into(),
            },
        );
    }
}

#[tokio::test]
async fn test_ticket() {
    let db = &prepare_mock_tickets();

    {
        let tickets = Query::get_all_tickets(db).await.unwrap();

        assert_eq!(tickets.len(), 2);
        for ticket in tickets {
            println!("{:?}", ticket);
        }
    }

    {
        let ticket_id = Uuid::new_v4().to_string();
        println!("ticket_id: {}", ticket_id);
        let ticket_time = get_timestamp();
        println!("ticket_time: {}", ticket_time);
        // let active_model: ticket::ActiveModel = ticket::Model {
        //     id: 2,
        //     ticket_id: ticket_id.clone(),
        //     ticket_type: TicketType::Normal,
        //     ticket_time,
        //     src_chain: "Bitcoin".to_string(),
        //     dst_chain: "EVM-Arbitrum".to_string(),
        //     action: TxAction::Transfer,
        //     token: "Bitcoin-RUNES-150:1".to_owned(),
        //     amount: 88888.to_string(),
        //     sender: Some("address_on_Bitcoin".to_string()),
        //     receiver: "address_on_Arbitrum".to_string(),
        //     memo: None,
        // }
        // .into();

        // let res = ticket::Entity::insert(active_model).exec(db).await;
        // println!("create_ticket result: {:?}", res);

        let ticket = Mutation::save_ticket(
            db,
            ticket::Model {
                ticket_id: ticket_id.clone(),
                ticket_type: TicketType::Normal,
                ticket_time,
                src_chain: "Bitcoin".to_string(),
                dst_chain: "EVM-Arbitrum".to_string(),
                action: TxAction::Transfer,
                token: "Bitcoin-RUNES-150:1".to_owned(),
                amount: 88888.to_string(),
                sender: Some("address_on_Bitcoin".to_string()),
                receiver: "address_on_Arbitrum".to_string(),
                memo: None,
            },
        )
        .await
        .unwrap();

        println!("{:?}", ticket);
        // assert_eq!(
        //     ticket,
        //     ticket::Model {
        //         ticket_id,
        //         ticket_type: TicketType::Normal,
        //         ticket_time,
        //         src_chain: "Bitcoin".to_string(),
        //         dst_chain: "EVM-Arbitrum".to_string(),
        //         action: TxAction::Transfer,
        //         token: "Bitcoin-RUNES-150:1".to_owned(),
        //         amount: 88888.to_string(),
        //         sender: Some("address_on_Bitcoin".to_string()),
        //         receiver: "address_on_Arbitrum".to_string(),
        //         memo: None,
        //     },
        // );
    }
}
