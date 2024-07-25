use super::m20240507_055143_one::{TicketType, TxAction};
use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(PendingTicket::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(PendingTicket::TicketId)
							.text()
							.not_null()
							.primary_key(),
					)
					.col(ColumnDef::new(PendingTicket::TicketSeq).big_unsigned())
					.col(
						ColumnDef::new(PendingTicket::TicketType)
							.not_null()
							.enumeration(
								Alias::new("ticket_type"),
								[TicketType::Normal, TicketType::Resubmit],
							),
					)
					.col(
						ColumnDef::new(PendingTicket::TicketTime)
							.big_unsigned()
							.not_null(),
					)
					.col(ColumnDef::new(PendingTicket::SrcChain).string().not_null())
					.col(ColumnDef::new(PendingTicket::DstChain).string().not_null())
					.col(
						ColumnDef::new(PendingTicket::Action)
							.not_null()
							.enumeration(
								Alias::new("tx_action"),
								[
									TxAction::Transfer,
									TxAction::Redeem,
									TxAction::Burn,
									TxAction::Mint,
								],
							),
					)
					.col(ColumnDef::new(PendingTicket::Token).string().not_null())
					.col(
						ColumnDef::new(PendingTicket::Amount)
							.big_unsigned()
							.not_null(),
					)
					.col(ColumnDef::new(PendingTicket::Sender).string().null())
					.col(ColumnDef::new(PendingTicket::Receiver).string().not_null())
					.col(ColumnDef::new(PendingTicket::Memo).binary().null())
					// .col(ColumnDef::new(Ticket::TxHash).string().not_null())
					.to_owned(),
			)
			.await?;

		// create index
		manager
			.create_index(
				Index::create()
					.if_not_exists()
					.name("pending-ticket_seq")
					.table(PendingTicket::Table)
					.col(PendingTicket::TicketSeq)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// drop index
		manager
			.drop_index(Index::drop().name("pending_ticket_seq").to_owned())
			.await?;
		// drop tables
		manager
			.drop_table(Table::drop().table(PendingTicket::Table).to_owned())
			.await?;
		// drop enum
		manager
			.drop_type(
				Type::drop()
					.if_exists()
					.names([
						SeaRc::new(TicketType::Type) as DynIden,
						SeaRc::new(TxAction::Type) as DynIden,
					])
					.to_owned(),
			)
			.await
	}
}

#[derive(DeriveIden)]
enum PendingTicket {
	Table,
	TicketId,
	TicketSeq,
	TicketType,
	TicketTime,
	SrcChain,
	DstChain,
	Action,
	Token,
	Amount,
	Sender,
	Receiver,
	Memo,
	// TxHash,
}
