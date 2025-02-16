pub mod cosmwasm;
pub mod evm;
pub mod icp;
pub mod solana;
pub mod sui;
pub mod ton;
use serde::{Deserialize, Serialize};

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
	Finalized { tx_hash: String },
	Unknown,
}
