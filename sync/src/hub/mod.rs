use crate::{
    types::{self, ChainMeta, OmnityTicket, Ticket, TokenMeta},
    with_omnity_canister,
};
use candid::{Decode, Encode};

use crate::service::{Mutation, Query};
use log::info;
use sea_orm::DbConn;
use std::error::Error;

use types::Error as OmnityError;
const FETCH_LIMIT: u64 = 50;
pub const CHAIN_SYNC_INTERVAL: u64 = 5;
pub const TOKEN_SYNC_INTERVAL: u64 = 5;
pub const TICKET_SYNC_INTERVAL: u64 = 3;

//full synchronization for chains
pub async fn sync_chains(db: &DbConn) -> Result<(), Box<dyn Error>> {
    with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
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
    with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
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
    with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
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
    with_omnity_canister("OMNITY_HUB_CANISTER_ID", |agent, canister_id| async move {
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
