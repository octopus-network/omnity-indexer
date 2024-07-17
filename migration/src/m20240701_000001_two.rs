use super::m20240507_055143_one::{ChainMeta, TokenMeta};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(TokenOnChain::Table)
					.col(ColumnDef::new(TokenOnChain::ChainId).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_chain_id")
							.from(TokenOnChain::Table, TokenOnChain::ChainId)
							.to(ChainMeta::Table, ChainMeta::ChainId),
					)
					.col(ColumnDef::new(TokenOnChain::TokenId).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_token_id")
							.from(TokenOnChain::Table, TokenOnChain::TokenId)
							.to(TokenMeta::Table, TokenMeta::TokenId),
					)
					.col(
						ColumnDef::new(TokenOnChain::Amount)
							.big_unsigned()
							.not_null(),
					)
					.primary_key(
						Index::create()
							.name("pk_chain_token")
							.col(TokenOnChain::ChainId)
							.col(TokenOnChain::TokenId)
							.primary(),
					)
					.to_owned(),
			)
			.await?;
		manager
			.create_table(
				Table::create()
					.table(TokenLedgerIdOnChain::Table)
					.col(
						ColumnDef::new(TokenLedgerIdOnChain::ChainId)
							.string()
							.not_null(),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_chain_id")
							.from(TokenLedgerIdOnChain::Table, TokenLedgerIdOnChain::ChainId)
							.to(ChainMeta::Table, ChainMeta::ChainId),
					)
					.col(
						ColumnDef::new(TokenLedgerIdOnChain::TokenId)
							.string()
							.not_null(),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_token_id")
							.from(TokenLedgerIdOnChain::Table, TokenLedgerIdOnChain::TokenId)
							.to(TokenMeta::Table, TokenMeta::TokenId),
					)
					.col(
						ColumnDef::new(TokenLedgerIdOnChain::ContractId)
							.string()
							.null(),
					)
					.primary_key(
						Index::create()
							.name("pk_chain_token_contract")
							.col(TokenLedgerIdOnChain::ChainId)
							.col(TokenLedgerIdOnChain::TokenId)
							.primary(),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(TokenOnChain::Table).to_owned())
			.await?;
		manager
			.drop_table(Table::drop().table(TokenLedgerIdOnChain::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum TokenOnChain {
	Table,
	ChainId,
	TokenId,
	Amount,
}

#[derive(DeriveIden)]
pub enum TokenLedgerIdOnChain {
	Table,
	ChainId,
	TokenId,
	ContractId,
}
