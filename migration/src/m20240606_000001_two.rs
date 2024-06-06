use sea_orm_migration::{prelude::*, sea_orm::EnumIter, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

// #[async_trait::async_trait]
// impl MigrationTrait for Migration {
	// async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
	// 	// create enum
	// 	manager
	// 		.create_type(
	// 			Type::create()
	// 				.as_enum(Alias::new("chain_type"))
	// 				.values([ChainType::SettlementChain, ChainType::ExecutionChain])
	// 				.to_owned(),
	// 		)
	// 		.await?;

	// 	// Create ChainMeta table
	// 	manager
	// 		.create_table(
	// 			Table::create()
	// 				.table(ChainMeta::Table)
	// 				.if_not_exists()
	// 				.col(
	// 					ColumnDef::new(ChainMeta::ChainId)
	// 						.string()
	// 						.not_null()
	// 						.primary_key(),
	// 				)
	// 				.col(ColumnDef::new(ChainMeta::CanisterId).text().not_null())
	// 				.col(ColumnDef::new(ChainMeta::ChainType).not_null().enumeration(
	// 					Alias::new("chain_type"),
	// 					[ChainType::SettlementChain, ChainType::ExecutionChain],
	// 				))
	// 				.col(
	// 					ColumnDef::new(ChainMeta::ChainState)
	// 						.not_null()
	// 						.enumeration(
	// 							Alias::new("chain_state"),
	// 							[ChainState::Active, ChainState::Deactive],
	// 						),
	// 				)
	// 				.col(ColumnDef::new(ChainMeta::ContractAddress).string().null())
	// 				.col(ColumnDef::new(ChainMeta::Counterparties).json().null())
	// 				.col(ColumnDef::new(ChainMeta::FeeToken).string().null())
	// 				.to_owned(),
	// 		)
	// 		.await?;

	// 	// create index
	// 	manager
	// 		.create_index(
	// 			Index::create()
	// 				.if_not_exists()
	// 				.name("idx-ticket_seq")
	// 				// .table(Ticket::Table)
	// 				// .col(Ticket::TicketSeq)
	// 				.to_owned(),
	// 		)
	// 		.await
	// }

	// async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// drop index
		// manager
		// 	.drop_index(Index::drop().name("idx-ticket_seq").to_owned())
		// 	.await?;
		// // Drop tables
		// manager
		// 	.drop_table(Table::drop().table(Ticket::Table).to_owned())
		// 	.await?;

		// drop emun
		// manager
		// 	.drop_type(
		// 		Type::drop()
		// 			.if_exists()
		// 			.names([
		// 				// SeaRc::new(ChainType::Type) as DynIden,
		// 			])
		// 			.to_owned(),
		// 	)
		// 	.await
	// }
// }

// #[derive(DeriveIden)]
// pub enum Directive {
// 	Table,
//     AddChain(Chain),
//     AddToken(Token),
//     UpdateChain(Chain),
//     UpdateToken(Token),
//     ToggleChainState(ToggleState),
//     UpdateFee(Factor),
// }