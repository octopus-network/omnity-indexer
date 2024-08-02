use sea_orm_migration::prelude::*;
use super::m20240507_055143_one::{TicketType, TxAction, TicketStatus};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// Create DeletedMintTicket table
		manager
			.create_table(
				Table::create()
					.table(DeletedMintTicket::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(DeletedMintTicket::TicketId)
							.text()
							.not_null()
							.primary_key(),
					)
					.col(ColumnDef::new(DeletedMintTicket::TicketSeq).big_unsigned())
					.col(ColumnDef::new(DeletedMintTicket::TicketType).not_null().enumeration(
						Alias::new("ticket_type"),
						[TicketType::Normal, TicketType::Resubmit],
					))
					.col(ColumnDef::new(DeletedMintTicket::TicketTime).big_unsigned().not_null())
					.col(ColumnDef::new(DeletedMintTicket::SrcChain).string().not_null())
					.col(ColumnDef::new(DeletedMintTicket::DstChain).string().not_null())
					.col(ColumnDef::new(DeletedMintTicket::Action).not_null().enumeration(
						Alias::new("tx_action"),
						[
							TxAction::Transfer,
							TxAction::Redeem,
							TxAction::Burn,
							TxAction::Mint,
						],
					))
					.col(ColumnDef::new(DeletedMintTicket::Token).string().not_null())
					.col(ColumnDef::new(DeletedMintTicket::Amount).big_unsigned().not_null())
					.col(ColumnDef::new(DeletedMintTicket::Sender).string().null())
					.col(ColumnDef::new(DeletedMintTicket::Receiver).string().not_null())
					.col(ColumnDef::new(DeletedMintTicket::Memo).binary().null())
					.col(ColumnDef::new(DeletedMintTicket::Status).not_null().enumeration(
						Alias::new("ticket_status"),
						[
							TicketStatus::Unknown,
							TicketStatus::WaitingForConfirmBySrc,
							TicketStatus::WaitingForConfirmByDest,
							TicketStatus::Finalized,
							TicketStatus::Pending,
						],
					))
					.col(ColumnDef::new(DeletedMintTicket::TxHash).string().not_null())
					.to_owned(),
			)
			.await?;

		// create index
		manager
			.create_index(
				Index::create()
					.if_not_exists()
					.name("idx-mint-ticket_seq")
					.table(DeletedMintTicket::Table)
					.col(DeletedMintTicket::TicketSeq)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// drop index
		manager
			.drop_index(Index::drop().name("idx-mint-ticket_seq").to_owned())
			.await?;
		// drop tables
		manager
			.drop_table(Table::drop().table(DeletedMintTicket::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
enum DeletedMintTicket {
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
	Status,
	TxHash,
}