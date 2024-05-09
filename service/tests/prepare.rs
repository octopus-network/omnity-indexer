use ::entity::{chain_meta, notes, sea_orm_active_enums::*, ticket, token_meta};
use sea_orm::*;
use serde_json::json;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;
pub fn get_timestamp() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis() as i64
}

#[cfg(feature = "mock")]
pub fn prepare_mock_notes() -> DatabaseConnection {
    MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            [notes::Model {
                id: 1,
                title: "Title A".to_owned(),
                text: "Text A".to_owned(),
            }],
            [notes::Model {
                id: 5,
                title: "Title C".to_owned(),
                text: "Text C".to_owned(),
            }],
            [notes::Model {
                id: 6,
                title: "Title D".to_owned(),
                text: "Text D".to_owned(),
            }],
            [notes::Model {
                id: 1,
                title: "Title A".to_owned(),
                text: "Text A".to_owned(),
            }],
            [notes::Model {
                id: 1,
                title: "New Title A".to_owned(),
                text: "New Text A".to_owned(),
            }],
            [notes::Model {
                id: 5,
                title: "Title C".to_owned(),
                text: "Text C".to_owned(),
            }],
        ])
        .append_exec_results([
            MockExecResult {
                last_insert_id: 6,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 6,
                rows_affected: 5,
            },
        ])
        .into_connection()
}

#[cfg(feature = "mock")]
pub fn prepare_mock_chains() -> DatabaseConnection {
    MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[
            chain_meta::Model {
                chain_id: "Bitcoin".to_owned(),
                chain_type: ChainType::SettlementChain,
                chain_state: ChainState::Active,
                canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_owned(),
                contract_address: None,
                counterparties: None,
                fee_token: None,
            },
            chain_meta::Model {
                chain_id: "Ethereum".to_owned(),
                chain_type: ChainType::SettlementChain,
                chain_state: ChainState::Active,
                canister_id: "bkyz2-fmaaa-aaaaa-qaaab-cai".to_owned(),
                contract_address: Some("Ethereum constract address".to_owned()),
                counterparties: Some(vec!["Bitcoin".to_owned()].into()),
                fee_token: None,
            },
            chain_meta::Model {
                chain_id: "ICP".to_owned(),
                chain_type: ChainType::ExecutionChain,
                chain_state: ChainState::Active,
                canister_id: "bkyz2-fmaaa-aaaaa-qadaab-cai".to_owned(),
                contract_address: Some("bkyz2-fmaaa-aaafa-qadaab-cai".to_owned()),
                counterparties: Some(vec!["Bitcoin".to_owned(), "Ethereum".to_owned()].into()),
                fee_token: Some("ICP".to_owned()),
            },
        ]])
        .append_exec_results([MockExecResult {
            last_insert_id: 4,
            rows_affected: 1,
        }])
        .into_connection()
}

#[cfg(feature = "mock")]
pub fn prepare_mock_tokens() -> DatabaseConnection {
    MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[
            token_meta::Model {
                token_id: "BTC".to_owned(),
                name: "BTC".to_owned(),
                symbol: "BTC".to_owned(),
                issue_chain: "Bitcoin".to_owned(),
                decimals: 18,
                icon: None,
                metadata: json!(HashMap::<String, String>::default()),
                dst_chains: vec![
                    "Ethereum".to_owned(),
                    "ICP".to_owned(),
                    "EVM-Arbitrum".to_owned(),
                    "EVM-Optimistic".to_owned(),
                    "EVM-Starknet".to_owned(),
                ]
                .into(),
            },
            token_meta::Model {
                token_id: "Bitcoin-RUNES-150:1".to_string(),
                name: "150:1".to_owned(),
                symbol: "150:1".to_owned(),
                issue_chain: "Bitcoin".to_string(),
                decimals: 18,
                icon: None,
                metadata: json!(HashMap::from([(
                    "rune_id".to_string(),
                    "150:1".to_string(),
                )])),
                dst_chains: vec![
                    "Ethereum".to_string(),
                    "ICP".to_string(),
                    "EVM-Arbitrum".to_string(),
                    "EVM-Optimistic".to_string(),
                    "EVM-Starknet".to_string(),
                ]
                .into(),
            },
            token_meta::Model {
                token_id: "ETH".to_string(),
                name: "ETH".to_owned(),
                symbol: "ETH".to_owned(),
                issue_chain: "Ethereum".to_string(),
                decimals: 18,
                icon: None,
                metadata: json!(HashMap::<String, String>::default()),
                dst_chains: vec![
                    "Bitcoin".to_string(),
                    "ICP".to_string(),
                    "EVM-Arbitrum".to_string(),
                    "EVM-Optimistic".to_string(),
                    "EVM-Starknet".to_string(),
                ]
                .into(),
            },
            token_meta::Model {
                token_id: "ICP".to_string(),
                name: "ICP".to_owned(),
                symbol: "ICP".to_owned(),
                issue_chain: "ICP".to_string(),
                decimals: 18,
                icon: None,
                metadata: serde_json::to_value(HashMap::<String, String>::default()).unwrap(),
                dst_chains: vec![
                    "Bitcoin".to_string(),
                    "Ethereum".to_string(),
                    "EVM-Arbitrum".to_string(),
                    "EVM-Optimistic".to_string(),
                    "EVM-Starknet".to_string(),
                ]
                .into(),
            },
        ]])
        .append_exec_results([MockExecResult {
            last_insert_id: 5,
            rows_affected: 1,
        }])
        .into_connection()
}

#[cfg(feature = "mock")]
pub fn prepare_mock_tickets() -> DatabaseConnection {
    MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[
            ticket::Model {
                ticket_id: Uuid::new_v4().to_string(),
                ticket_type: TicketType::Normal,
                ticket_time: get_timestamp(),
                src_chain: "Bitcoin".to_string(),
                dst_chain: "EVM-Arbitrum".to_string(),
                action: TxAction::Transfer,
                token: "Bitcoin-RUNES-150:1".to_owned(),
                amount: 88888.to_string(),
                sender: Some("address_on_Bitcoin".to_string()),
                receiver: "address_on_Arbitrum".to_string(),
                memo: None,
            },
            ticket::Model {
                ticket_id: Uuid::new_v4().to_string(),
                ticket_type: TicketType::Normal,
                ticket_time: get_timestamp(),
                src_chain: "EVM-Arbitrum".to_string(),
                dst_chain: "Bitcoin".to_string(),
                action: TxAction::Redeem,
                token: "Bitcoin-RUNES-150:1".to_owned(),
                amount: 88888.to_string(),
                sender: Some("address_on_Arbitrum".to_string()),
                receiver: "address_on_Bitcoin".to_string(),
                memo: None,
            },
        ]])
        .append_exec_results([MockExecResult {
            last_insert_id: 3,
            rows_affected: 1,
        }])
        .into_connection()
}
