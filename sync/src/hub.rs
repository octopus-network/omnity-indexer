use crate::{
    get_effective_canister_id,
    types::{self, Chain, ChainMeta, Ticket, TokenMeta},
    with_agent, with_agent_as,
};
use candid::{Decode, Encode};
use ic_agent::{export::Principal, Agent, Identity};
use ic_utils::interfaces::ManagementCanister;
use omnity_indexer_service::{Mutation, Query};
use sea_orm::DbConn;
use std::{error::Error, future::Future, path::Path};
use types::{ChainState, ChainType, Error as OmnityError};

pub fn with_omnity_hub_canister<F, R>(f: F)
where
    R: Future<Output = Result<(), Box<dyn Error>>>,
    F: FnOnce(Agent, Principal) -> R,
{
    with_agent(|agent| async move {
        let canister_id = create_omnity_hub_canister(&agent).await?;
        f(agent, canister_id).await
    })
}

//TODO: add env var for identity
pub fn with_omnity_hub_canister_as<I, F, R>(identity: I, f: F)
where
    I: Identity + 'static,
    R: Future<Output = Result<(), Box<dyn Error>>>,
    F: FnOnce(Agent, Principal) -> R,
{
    with_agent_as(identity, |agent| async move {
        let canister_id = create_omnity_hub_canister(&agent).await?;
        f(agent, canister_id).await
    })
}

pub async fn create_omnity_hub_canister(agent: &Agent) -> Result<Principal, Box<dyn Error>> {
    //TODO: add env var for canister_id
    match std::env::var("OMNITY_HUB_CANISTER_ID") {
        Ok(canister_id) => {
            println!("hub canister_id: {:?}", canister_id);
            Ok(Principal::from_text(canister_id)?)
        }
        Err(e) => {
            eprintln!(
                "Could not find the OMNITY_HUB_CANISTER_ID environment variable: {:?}",
                e
            );
            let canister_env = std::env::var("OMNITY_HUB_CANISTER_PATH")
                .expect("Need to specify the OMNITY_HUB_CANISTER_PATH environment variable.");

            let canister_path = Path::new(&canister_env);

            let canister_wasm = if !canister_path.exists() {
                panic!("Could not find the omnity hub canister WASM file.");
            } else {
                std::fs::read(canister_path).expect("Could not read file.")
            };

            let ic00 = ManagementCanister::create(agent);

            let (canister_id,) = ic00
                .create_canister()
                .as_provisional_create_with_amount(None)
                .with_effective_canister_id(get_effective_canister_id())
                .call_and_wait()
                .await?;

            ic00.install_code(&canister_id, &canister_wasm)
                .with_raw_arg(vec![])
                .call_and_wait()
                .await?;

            Ok(canister_id)
        }
    }
}

pub fn sync_chains(db: &DbConn) {
    with_omnity_hub_canister(|agent, canister_id| async move {
        let offset = 0u64;
        let limit = 10u64;
        let args = Encode!(&offset, &limit).unwrap();
        let ret = agent
            .query(&canister_id, "get_chain_metas")
            .with_arg(args)
            .call()
            .await?;

        let ret = Decode!(&ret, Result<Vec<ChainMeta>, OmnityError>)?.unwrap();

        for chain in ret {
            Mutation::create_chain(db, chain.into()).await?;
        }

        Ok(())
    })
}

pub fn sync_tokens(db: &DbConn) {
    with_omnity_hub_canister(|agent, canister_id| async move {
        let offset = 0u64;
        let limit = 10u64;
        let args = Encode!(&offset, &limit).unwrap();
        let ret = agent
            .query(&canister_id, "get_token_metas")
            .with_arg(args)
            .call()
            .await?;

        let ret = Decode!(&ret, Result<Vec<TokenMeta>, OmnityError>)?.unwrap();

        for chain in ret {
            Mutation::create_token(db, chain.into()).await?;
        }

        Ok(())
    })
}

pub fn sync_tickets(db: &DbConn) {
    with_omnity_hub_canister(|agent, canister_id| async move {
        let offset = 0u64;
        let limit = 10u64;
        let args = Encode!(&offset, &limit).unwrap();
        let ret = agent
            .query(&canister_id, "get_txs")
            .with_arg(args)
            .call()
            .await?;

        let tickets = Decode!(&ret, Result<Vec<Ticket>, OmnityError>)?.unwrap();

        for ticket in tickets {
            Mutation::create_ticket(db, ticket.into()).await?;
        }

        Ok(())
    })
}
