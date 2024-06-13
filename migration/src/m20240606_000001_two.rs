use super::m20240507_055143_one::ChainMeta;
use sea_orm_migration::{prelude::*, sea_orm::EnumIter, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// create enum
		manager
			.create_type(
				Type::create()
					.as_enum(Alias::new("add_token"))
					.values([
						AddToken::TokenId,
						AddToken::Name,
						AddToken::Symbol,
						AddToken::Decimals,
						AddToken::Icon,
						AddToken::Metadata,
					])
					.to_owned(),
			)
			.await?;
		manager
			.create_type(
				Type::create()
					.as_enum(Alias::new("update_token"))
					.values([
						AddToken::TokenId,
						AddToken::Name,
						AddToken::Symbol,
						AddToken::Decimals,
						AddToken::Icon,
						AddToken::Metadata,
					])
					.to_owned(),
			)
			.await?;
		manager
			.create_type(
				Type::create()
					.as_enum(Alias::new("toggle_chain_state"))
					.values([ToggleChainState::ChainId, ToggleChainState::ToggleAction])
					.to_owned(),
			)
			.await?;

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
					.col(ColumnDef::new(Directive::AddChain).json().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk-chain-meta")
							.from(Directive::Table, Directive::AddChain)
							.to(ChainMeta::Table, ChainMeta::Table),
					)
					.col(ColumnDef::new(Directive::AddToken).not_null().enumeration(
						Alias::new("add_token"),
						[
							AddToken::TokenId,
							AddToken::Name,
							AddToken::Symbol,
							AddToken::Decimals,
							AddToken::Icon,
							AddToken::Metadata,
						],
					))
					.col(ColumnDef::new(Directive::UpdateChain).json().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fk-update-chain")
							.from(Directive::Table, Directive::UpdateChain)
							.to(ChainMeta::Table, ChainMeta::Table),
					)
					.col(
						ColumnDef::new(Directive::UpdateToken)
							.not_null()
							.enumeration(
								Alias::new("update_token"),
								[
									UpdateToken::TokenId,
									UpdateToken::Name,
									UpdateToken::Symbol,
									UpdateToken::Decimals,
									UpdateToken::Icon,
									UpdateToken::Metadata,
								],
							),
					)
					.col(
						ColumnDef::new(Directive::ToggleChainState)
							.not_null()
							.enumeration(
								Alias::new("toggle_chain_state"),
								[ToggleChainState::ChainId, ToggleChainState::ToggleAction],
							),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk-toggle-action")
							.from(ToggleChainState::Type, ToggleChainState::ToggleAction)
							.to(ChainMeta::Table, ChainMeta::ChainState),
					)
					.col(ColumnDef::new(Directive::UpdateFee).json().not_null())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// drop tables
		manager
			.drop_table(Table::drop().table(Directive::Table).to_owned())
			.await?;
		// drop enums
		manager
			.drop_type(
				Type::drop()
					.if_exists()
					.names([
						SeaRc::new(AddToken::Type) as DynIden,
						SeaRc::new(UpdateToken::Type) as DynIden,
						SeaRc::new(ToggleChainState::Type) as DynIden,
					])
					.to_owned(),
			)
			.await
	}
}

#[derive(Iden)]
pub enum Directive {
	Table,
	DirectiveSeq,
	AddChain,
	AddToken,
	UpdateChain,
	UpdateToken,
	ToggleChainState,
	UpdateFee,
}

#[derive(Iden, EnumIter)]
pub enum AddToken {
	#[iden = "add_token"]
	Type,
	#[iden = "TokenId"]
	TokenId,
	#[iden = "Name"]
	Name,
	#[iden = "Symbol"]
	Symbol,
	#[iden = "Decimals"]
	Decimals,
	#[iden = "Icon"]
	Icon,
	#[iden = "Metadata"]
	Metadata,
}

#[derive(Iden, EnumIter)]
pub enum UpdateToken {
	#[iden = "update_token"]
	Type,
	#[iden = "TokenId"]
	TokenId,
	#[iden = "Name"]
	Name,
	#[iden = "Symbol"]
	Symbol,
	#[iden = "Decimals"]
	Decimals,
	#[iden = "Icon"]
	Icon,
	#[iden = "Metadata"]
	Metadata,
}

#[derive(Iden, EnumIter)]
pub enum ToggleChainState {
	#[iden = "toggle_chain_state"]
	Type,
	#[iden = "ChainId"]
	ChainId,
	#[iden = "ToggleAction"]
	ToggleAction,
}
