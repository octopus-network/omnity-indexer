use crate::{
    read_config,
    types::{self, ChainMeta, OmnityTicket, Ticket, TokenMeta},
    with_agent, with_agent_as,
};
use candid::{Decode, Encode};
use ic_agent::{export::Principal, Agent, Identity};

use crate::service::{Mutation, Query};
use log::info;
use sea_orm::DbConn;
use std::{error::Error, future::Future};

use types::Error as OmnityError;
const FETCH_LIMIT: u64 = 50;
pub const CHAIN_SYNC_INTERVAL: u64 = 5;
pub const TOKEN_SYNC_INTERVAL: u64 = 5;
pub const TICKET_SYNC_INTERVAL: u64 = 3;

pub async fn with_omnity_hub_canister<F, R>(f: F) -> Result<(), Box<dyn Error>>
where
    R: Future<Output = Result<(), Box<dyn Error>>>,
    F: FnOnce(Agent, Principal) -> R,
{
    with_agent(|agent| async move {
        let canister_id = create_omnity_hub_canister().await?;
        f(agent, canister_id).await
    })
    .await
}

pub async fn with_omnity_hub_canister_as<I, F, R>(identity: I, f: F) -> Result<(), Box<dyn Error>>
where
    I: Identity + 'static,
    R: Future<Output = Result<(), Box<dyn Error>>>,
    F: FnOnce(Agent, Principal) -> R,
{
    with_agent_as(identity, |agent| async move {
        let canister_id = create_omnity_hub_canister().await?;
        f(agent, canister_id).await
    })
    .await
}

pub async fn create_omnity_hub_canister() -> Result<Principal, Box<dyn Error>> {
    match std::env::var("OMNITY_HUB_CANISTER_ID") {
        Ok(hub_canister_id) => {
            info!("get hub canister id from env var :{}", hub_canister_id);
            Ok(Principal::from_text(hub_canister_id)?)
        }

        Err(_) => {
            let hub_canister_id = read_config(|c| c.omnity_hub_canister_id.to_owned());
            info!("get hub canister id from  config file :{hub_canister_id:?}");
            Ok(Principal::from_text(hub_canister_id)?)
        }
    }
}

//full synchronization for chains
pub async fn sync_chains(db: &DbConn) -> Result<(), Box<dyn Error>> {
    with_omnity_hub_canister(|agent, canister_id| async move {
        info!("{:?} syncing chains ... ", chrono::Utc::now());
        let args: Vec<u8> = Encode!(&Vec::<u8>::new())?;
        let ret = agent
            .query(&canister_id, "get_chain_size")
            .with_arg(args)
            .call()
            .await?;
        let chain_size = Decode!(&ret, Result<u64, OmnityError>)??;
        info!("chain size: {:?}", chain_size);

        let mut from_seq = 0u64;
        while from_seq < chain_size {
            let args = Encode!(&from_seq, &FETCH_LIMIT)?;
            let ret = agent
                .query(&canister_id, "get_chain_metas")
                .with_arg(args)
                .call()
                .await?;
            let chains: Vec<ChainMeta> = Decode!(&ret, Result<Vec<ChainMeta>, OmnityError>)??;
            info!("sync chains from offset: {}", from_seq);
            for chain in chains.iter() {
                Mutation::save_chain(db, chain.clone().into()).await?;
            }
            from_seq += chains.len() as u64;
            if chains.is_empty() {
                break;
            }
        }
        Ok(())
    })
    .await
}

//full synchronization for tokens
pub async fn sync_tokens(db: &DbConn) -> Result<(), Box<dyn Error>> {
    with_omnity_hub_canister(|agent, canister_id| async move {
        info!("{:?} syncing tokens ... ", chrono::Utc::now());

        let args: Vec<u8> = Encode!(&Vec::<u8>::new())?;
        let ret = agent
            .query(&canister_id, "get_token_size")
            .with_arg(args)
            .call()
            .await?;
        let token_size = Decode!(&ret, Result<u64, OmnityError>)??;
        info!("total token size: {:?}", token_size);

        let mut offset = 0u64;
        while offset < token_size {
            let args = Encode!(&offset, &FETCH_LIMIT)?;
            let ret = agent
                .query(&canister_id, "get_token_metas")
                .with_arg(args)
                .call()
                .await?;
            let tokens: Vec<TokenMeta> = Decode!(&ret, Result<Vec<TokenMeta>, OmnityError>)??;
            info!("total tokens from offset: {} ", offset);
            for token in tokens.iter() {
                Mutation::save_token(db, token.clone().into()).await?;
            }
            offset += tokens.len() as u64;
            if tokens.is_empty() {
                break;
            }
        }

        Ok(())
    })
    .await
}

pub async fn send_tickets(ticket: types::Ticket) -> Result<(), Box<dyn Error>> {
    with_omnity_hub_canister(|agent, canister_id| async move {
        info!("{:?} send tickets to hub... ", chrono::Utc::now());

        let args: Vec<u8> = Encode!(&ticket)?;
        let ret = agent
            .update(&canister_id, "send_ticket")
            .with_arg(args)
            .call_and_wait()
            .await?;
        let ret = Decode!(&ret, Result<(), OmnityError>)??;
        info!("send ticket result: {:?}", ret);

        Ok(())
    })
    .await
}

//increment synchronization for tickets
pub async fn sync_tickets(db: &DbConn) -> Result<(), Box<dyn Error>> {
    with_omnity_hub_canister(|agent, canister_id| async move {
        info!("{:?} syncing tickets from hub ... ", chrono::Utc::now());

        let args: Vec<u8> = Encode!(&Vec::<u8>::new())?;
        let ret = agent
            .query(&canister_id, "sync_ticket_size")
            .with_arg(args)
            .call()
            .await?;
        let ticket_size = Decode!(&ret, Result<u64, OmnityError>)??;
        info!("total ticket size: {:?}", ticket_size);

        //get latest ticket seq from  postgresql database
        let latest_ticket_seq = Query::get_latest_ticket(db).await?.map(|t| {
            info!("latest ticket : {:?}", t);
            t.ticket_seq
        });
        let offset = match latest_ticket_seq {
            Some(t) => {
                info!("latest ticket seq: {:?}", t);
                // the latest ticket seq may be Some or may be None
                t.map_or(0u64, |t| (t + 1) as u64)
            }
            None => {
                info!("no tickets found");
                0u64
            }
        };

        let tickets_to_fetch = ticket_size.saturating_sub(offset);
        info!("need to fetch tickets size: {:?}", tickets_to_fetch);

        let mut limit = FETCH_LIMIT;
        for next_offset in (offset..ticket_size).step_by(limit as usize) {
            info!("next_offset: {:?}", next_offset);
            limit = std::cmp::min(limit, ticket_size - next_offset);
            let args = Encode!(&next_offset, &limit)?;
            let ret = agent
                .query(&canister_id, "sync_tickets")
                .with_arg(args)
                .call()
                .await?;
            let new_tickets = Decode!(&ret, Result<Vec<(u64, OmnityTicket)>, OmnityError>)??;
            // info!("synced tickets {:?} ", new_tickets);
            for (seq, ticket) in new_tickets.iter() {
                let ticket_modle = Ticket::from_omnity_ticket(*seq, ticket.clone()).into();
                Mutation::save_ticket(db, ticket_modle).await?;
            }
            if new_tickets.len() < limit as usize {
                break;
            }
        }
        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {

    use crate::service::Query;
    use crate::{get_timestamp, random_txid, types, Database};
    use dotenvy::dotenv;

    use super::*;
    use log::info;
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
        let db_url = std::env::var("DATABASE_URL").unwrap();
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
        let db_url = std::env::var("DATABASE_URL").unwrap();
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
        let db_url = std::env::var("DATABASE_URL").unwrap();
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
        let db_url = std::env::var("DATABASE_URL").unwrap();
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
