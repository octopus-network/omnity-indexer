use super::m20240507_055143_one::ChainMeta;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(BridgeFeeLog::Table)
					.col(ColumnDef::new(BridgeFeeLog::ChainId).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_log_chain_id")
							.from(BridgeFeeLog::Table, BridgeFeeLog::ChainId)
							.to(ChainMeta::Table, ChainMeta::ChainId),
					)
					.col(ColumnDef::new(BridgeFeeLog::Date).string().not_null())
					.col(ColumnDef::new(BridgeFeeLog::FeeTokenId).string().not_null())
					.col(ColumnDef::new(BridgeFeeLog::Amount).string().not_null())
					.col(
						ColumnDef::new(BridgeFeeLog::TotalTicket)
							.integer()
							.not_null(),
					)
					.col(ColumnDef::new(BridgeFeeLog::Seqs).string().not_null())
					.primary_key(
						Index::create()
							.name("pk_bridge_fee_log")
							.col(BridgeFeeLog::ChainId)
							.col(BridgeFeeLog::Date)
							.primary(),
					)
					.to_owned(),
			)
			.await
	}
}

#[derive(DeriveIden)]
pub enum BridgeFeeLog {
	Table,
	ChainId,
	Date,
	FeeTokenId,
	Amount,
	TotalTicket,
	Seqs,
}
