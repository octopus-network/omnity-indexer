use sea_orm_migration::{prelude::*, sea_orm::EnumIter, sea_query::extension::postgres::Type};
use super::m20240507_055143_one::ChainMeta;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// Create ChainMeta table
		manager
			.create_table(
				Table::create()
					.table(Directive::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Directive::DirectiveSeq)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(
						ColumnDef::new(Directive::AddChain(ChainMeta))
						.not_null()
						.enumeration(Alias::new("chain_meta"), ChainMeta::iter()))
					.to_owned(),
			)
			.await?;
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
			manager
			.drop_table(Table::drop().table(Directive::Table).to_owned())
			.await?;
	}
}

#[derive(Iden)]
pub enum Directive {
	Table,
	DirectiveSeq,
    AddChain(ChainMeta),
    // AddToken(Token),
    // UpdateChain(ChainMeta),
    // UpdateToken(Token),
    // ToggleChainState(ToggleState),
    // UpdateFee(Factor),
}

#[derive(Iden, EnumIter)]
pub enum Token {
	TokenId,
	Name,
	Symbol,
	Decimals,
	Icon,
	Metadata,
}