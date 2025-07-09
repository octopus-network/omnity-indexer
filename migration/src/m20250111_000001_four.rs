use super::m20240507_055143_one::{ChainMeta, LaunchPad, TokenMeta};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	// async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
	// 	manager
	// 		.create_table(
	// 			Table::create()
	// 				.table(BridgeFeeLog::Table)
	// 				.col(ColumnDef::new(BridgeFeeLog::ChainId).string().not_null())
	// 				.foreign_key(
	// 					ForeignKey::create()
	// 						.name("fk_log_chain_id")
	// 						.from(BridgeFeeLog::Table, BridgeFeeLog::ChainId)
	// 						.to(ChainMeta::Table, ChainMeta::ChainId),
	// 				)
	// 				.col(ColumnDef::new(BridgeFeeLog::Date).string().not_null())
	// 				.col(ColumnDef::new(BridgeFeeLog::FeeTokenId).string().not_null())
	// 				.col(ColumnDef::new(BridgeFeeLog::Amount).string().not_null())
	// 				.col(
	// 					ColumnDef::new(BridgeFeeLog::TotalTicket)
	// 						.integer()
	// 						.not_null(),
	// 				)
	// 				.col(ColumnDef::new(BridgeFeeLog::Seqs).string().not_null())
	// 				.primary_key(
	// 					Index::create()
	// 						.name("pk_bridge_fee_log")
	// 						.col(BridgeFeeLog::ChainId)
	// 						.col(BridgeFeeLog::Date)
	// 						.primary(),
	// 				)
	// 				.to_owned(),
	// 		)
	// 		.await
	// }

	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(LaunchPad::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(LaunchPad::LaunchPad)
							.string()
							.not_null()
							.primary_key(),
					)
					.col(ColumnDef::new(LaunchPad::CainisterId).string().not_null())
					.to_owned(),
			)
			.await?;
		manager
			.alter_table(
				Table::alter()
					.table(TokenMeta::Table)
					.add_column_if_not_exists(ColumnDef::new(TokenMeta::LaunchPad).string().null())
					.add_foreign_key(
						&TableForeignKey::new()
							.name("fk_launch_pad")
							.from_tbl(TokenMeta::Table)
							.from_col(TokenMeta::LaunchPad)
							.to_tbl(LaunchPad::Table)
							.to_col(LaunchPad::LaunchPad)
							.to_owned(),
					)
					.to_owned(),
			)
			.await
	}
}

// #[derive(DeriveIden)]
// pub enum BridgeFeeLog {
// 	Table,
// 	ChainId,
// 	Date,
// 	FeeTokenId,
// 	Amount,
// 	TotalTicket,
// 	Seqs,
// }
