use super::m20240507_055143_one::{TicketStatus, TicketType, TokenMeta, TxAction};
use sea_orm_migration::prelude::*;

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
					.col(
						ColumnDef::new(DeletedMintTicket::TicketType)
							.not_null()
							.enumeration(
								Alias::new("ticket_type"),
								[TicketType::Normal, TicketType::Resubmit],
							),
					)
					.col(
						ColumnDef::new(DeletedMintTicket::TicketTime)
							.big_unsigned()
							.not_null(),
					)
					.col(
						ColumnDef::new(DeletedMintTicket::SrcChain)
							.string()
							.not_null(),
					)
					.col(
						ColumnDef::new(DeletedMintTicket::DstChain)
							.string()
							.not_null(),
					)
					.col(
						ColumnDef::new(DeletedMintTicket::Action)
							.not_null()
							.enumeration(
								Alias::new("tx_action"),
								[
									TxAction::Transfer,
									TxAction::Redeem,
									TxAction::Burn,
									TxAction::Mint,
									TxAction::RedeemIcpChainKeyAssets,
								],
							),
					)
					.col(ColumnDef::new(DeletedMintTicket::Token).string().not_null())
					.col(
						ColumnDef::new(DeletedMintTicket::Amount)
							.string()
							.not_null(),
					)
					.col(ColumnDef::new(DeletedMintTicket::Sender).string().null())
					.col(
						ColumnDef::new(DeletedMintTicket::Receiver)
							.string()
							.not_null(),
					)
					.col(ColumnDef::new(DeletedMintTicket::Memo).string().null())
					.col(
						ColumnDef::new(DeletedMintTicket::Status)
							.not_null()
							.enumeration(
								Alias::new("ticket_status"),
								[
									TicketStatus::Unknown,
									TicketStatus::WaitingForConfirmBySrc,
									TicketStatus::WaitingForConfirmByDest,
									TicketStatus::Finalized,
									TicketStatus::Pending,
									TicketStatus::Failed,
								],
							),
					)
					.col(ColumnDef::new(DeletedMintTicket::TxHash).string().null())
					.col(ColumnDef::new(DeletedMintTicket::Date).string().not_null())
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(PendingTicket::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(PendingTicket::TicketIndex)
							.integer()
							.auto_increment()
							.primary_key(),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(TokenVolume::Table)
					.col(
						ColumnDef::new(TokenVolume::TokenId)
							.string()
							.not_null()
							.primary_key(),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_token_id_volume")
							.from(TokenVolume::Table, TokenVolume::TokenId)
							.to(TokenMeta::Table, TokenMeta::TokenId),
					)
					.col(ColumnDef::new(TokenVolume::TicketCount).string().not_null())
					.col(
						ColumnDef::new(TokenVolume::HistoricalVolume)
							.string()
							.not_null(),
					)
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
}

#[derive(DeriveIden)]
pub enum DeletedMintTicket {
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
	Date,
}

#[derive(DeriveIden)]
enum PendingTicket {
	Table,
	TicketIndex,
}

#[derive(DeriveIden)]
enum TokenVolume {
	Table,
	TokenId,
	TicketCount,
	HistoricalVolume,
}
