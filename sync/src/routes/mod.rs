pub mod cosmwasm;
pub mod evm;
pub mod icp;
pub mod solana;
pub mod sui;
pub mod ton;
use serde::{Deserialize, Serialize};

pub const TOKEN_LEDGER_ID_ON_CHAIN_SYNC_INTERVAL: u64 = 1800;

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
	Finalized { tx_hash: String },
	Unknown,
}
