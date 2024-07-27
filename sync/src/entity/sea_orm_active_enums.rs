//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "chain_state")]
pub enum ChainState {
	#[sea_orm(string_value = "Active")]
	Active,
	#[sea_orm(string_value = "Deactive")]
	Deactive,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "chain_type")]
pub enum ChainType {
	#[sea_orm(string_value = "ExecutionChain")]
	ExecutionChain,
	#[sea_orm(string_value = "SettlementChain")]
	SettlementChain,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ticket_status")]
pub enum TicketStatus {
	#[sea_orm(string_value = "Finalized")]
	Finalized,
	#[sea_orm(string_value = "Unknown")]
	Unknown,
	#[sea_orm(string_value = "WaitingForConfirmByDest")]
	WaitingForConfirmByDest,
	#[sea_orm(string_value = "WaitingForConfirmBySrc")]
	WaitingForConfirmBySrc,
	#[sea_orm(string_value = "Pending")]
	Pending,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ticket_type")]
pub enum TicketType {
	#[sea_orm(string_value = "Normal")]
	Normal,
	#[sea_orm(string_value = "Resubmit")]
	Resubmit,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tx_action")]
pub enum TxAction {
	#[sea_orm(string_value = "Redeem")]
	Redeem,
	#[sea_orm(string_value = "Transfer")]
	Transfer,
	#[sea_orm(string_value = "Burn")]
	Burn,
	#[sea_orm(string_value = "Mint")]
	Mint,
}
