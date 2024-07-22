use crate::entity::chain_meta;
use crate::entity::chain_meta::Entity as ChainMeta;
use crate::entity::sea_orm_active_enums::{TicketStatus, TxAction};
use crate::entity::ticket;
use crate::entity::ticket::Entity as Ticket;
use crate::entity::token_ledger_id_on_chain;
use crate::entity::token_ledger_id_on_chain::Entity as TokenLedgerIdOnChain;
use crate::entity::token_meta;
use crate::entity::token_meta::Entity as TokenMeta;
use crate::entity::token_on_chain;
use crate::entity::token_on_chain::Entity as TokenOnChain;
use crate::TxHash;
use log::info;
use sea_orm::sea_query::OnConflict;
use sea_orm::*;

pub struct Query;

impl Query {
	pub async fn get_all_chains(db: &DbConn) -> Result<Vec<chain_meta::Model>, DbErr> {
		ChainMeta::find().all(db).await
	}
	pub async fn get_all_tokens(db: &DbConn) -> Result<Vec<token_meta::Model>, DbErr> {
		TokenMeta::find().all(db).await
	}
	pub async fn get_all_tickets(db: &DbConn) -> Result<Vec<ticket::Model>, DbErr> {
		Ticket::find().all(db).await
	}
	pub async fn get_all_tokens_on_chains(
		db: &DbConn,
	) -> Result<Vec<token_on_chain::Model>, DbErr> {
		TokenOnChain::find().all(db).await
	}
	pub async fn get_ticket_by_id(
		db: &DbConn,
		ticket_id: String,
	) -> Result<Option<ticket::Model>, DbErr> {
		Ticket::find_by_id(ticket_id).one(db).await
	}
	pub async fn get_token_on_chain_by_id(
		db: &DbConn,
		chain_id: String,
		token_id: String,
	) -> Result<Option<token_on_chain::Model>, DbErr> {
		TokenOnChain::find_by_id((chain_id, token_id)).one(db).await
	}
	pub async fn get_token_ledger_id_on_chain_by_id(
		db: &DbConn,
		chain_id: String,
		token_id: String,
	) -> Result<Option<token_ledger_id_on_chain::Model>, DbErr> {
		TokenLedgerIdOnChain::find_by_id((chain_id, token_id))
			.one(db)
			.await
	}
	pub async fn get_latest_ticket(db: &DbConn) -> Result<Option<ticket::Model>, DbErr> {
		Ticket::find()
			.filter(ticket::Column::TicketSeq.is_not_null())
			.order_by_desc(ticket::Column::TicketSeq)
			.one(db)
			.await
	}
	pub async fn get_unconfirmed_tickets(
		db: &DbConn,
		dest: String,
	) -> Result<Vec<ticket::Model>, DbErr> {
		Ticket::find()
			.filter(
				Condition::all()
					// The ticket is not finalized
					.add(ticket::Column::Status.ne(TicketStatus::Finalized))
					// The ticket's destination chain matches `dest`
					.add(ticket::Column::DstChain.eq(dest)),
			)
			.all(db)
			.await
	}

	pub async fn get_confirmed_tickets(
		db: &DbConn,
		dest: String,
	) -> Result<Vec<ticket::Model>, DbErr> {
		Ticket::find()
			.filter(
				Condition::all()
					.add(ticket::Column::Status.eq(TicketStatus::Finalized))
					.add(ticket::Column::DstChain.eq(dest))
					.add(ticket::Column::TxHash.contains("0")),
			)
			.all(db)
			.await
	}

	pub async fn get_non_updated_mint_tickets(db: &DbConn) -> Result<Vec<ticket::Model>, DbErr> {
		Ticket::find()
			.filter(
				Condition::all()
					// The ticket is for minting action
					.add(ticket::Column::Action.eq(TxAction::Mint))
					// The ticket amount is not updated yet
					.add(ticket::Column::Amount.eq(0)),
			)
			.all(db)
			.await
	}

	pub async fn get_updated_mint_tickets(db: &DbConn) -> Result<Vec<ticket::Model>, DbErr> {
		Ticket::find()
			.filter(
				Condition::all()
					// The ticket is for minting action
					.add(ticket::Column::Action.eq(TxAction::Mint))
					// The ticket amount is updated
					.add(ticket::Column::Amount.ne(0)),
			)
			.all(db)
			.await
	}

	pub async fn get_null_sender_tickets(db: &DbConn) -> Result<Vec<ticket::Model>, DbErr> {
		Ticket::find()
			.filter(ticket::Column::Sender.is_null())
			.order_by_desc(ticket::Column::TicketSeq)
			.all(db)
			.await
	}
}

pub struct Delete;

impl Delete {
	pub async fn remove_ticket_by_id(
		db: &DbConn,
		ticket_id: String,
	) -> Result<DeleteResult, DbErr> {
		Ticket::delete_by_id(ticket_id).exec(db).await
	}
}

pub struct Mutation;

impl Mutation {
	pub async fn save_all_token_ledger_id_on_chain(
		db: &DbConn,
		token_ledger_id_on_chain: token_ledger_id_on_chain::Model,
	) -> Result<token_ledger_id_on_chain::Model, DbErr> {
		let active_model: token_ledger_id_on_chain::ActiveModel =
			token_ledger_id_on_chain.clone().into();
		let on_conflict = OnConflict::columns([
			token_ledger_id_on_chain::Column::ChainId,
			token_ledger_id_on_chain::Column::TokenId,
		])
		.do_nothing()
		.to_owned();
		let insert_result = TokenLedgerIdOnChain::insert(active_model.clone())
			.on_conflict(on_conflict)
			.exec(db)
			.await;

		match insert_result {
			Ok(ret) => {
				info!("insert token ledger id result : {:?}", ret);
			}
			Err(_) => {
				info!("the token ledger id already exited, need to update it !");

				let res = TokenLedgerIdOnChain::update(active_model)
					.filter(
						Condition::all()
							.add(
								token_ledger_id_on_chain::Column::ChainId
									.eq(token_ledger_id_on_chain.chain_id.to_owned()),
							)
							.add(
								token_ledger_id_on_chain::Column::TokenId
									.eq(token_ledger_id_on_chain.token_id.to_owned()),
							),
					)
					.exec(db)
					.await
					.map(|token_on_chain| token_on_chain);
				info!("update token ledger id result : {:?}", res);
			}
		}
		Ok(token_ledger_id_on_chain::Model {
			..token_ledger_id_on_chain
		})
	}

	pub async fn save_token_on_chain(
		db: &DbConn,
		token_on_chain: token_on_chain::Model,
	) -> Result<token_on_chain::Model, DbErr> {
		let active_model: token_on_chain::ActiveModel = token_on_chain.clone().into();
		let on_conflict = OnConflict::columns([
			token_on_chain::Column::ChainId,
			token_on_chain::Column::TokenId,
		])
		.do_nothing()
		.to_owned();
		let insert_result = TokenOnChain::insert(active_model.clone())
			.on_conflict(on_conflict)
			.exec(db)
			.await;

		match insert_result {
			Ok(ret) => {
				info!("insert token on chain result : {:?}", ret);
			}
			Err(_) => {
				info!("the token on chain already exited, need to update it !");

				let res = TokenOnChain::update(active_model)
					.filter(
						Condition::all()
							.add(
								token_on_chain::Column::ChainId
									.eq(token_on_chain.chain_id.to_owned()),
							)
							.add(
								token_on_chain::Column::TokenId
									.eq(token_on_chain.token_id.to_owned()),
							),
					)
					.exec(db)
					.await
					.map(|token_on_chain| token_on_chain);
				info!("update token on chain result : {:?}", res);
			}
		}
		Ok(token_on_chain::Model { ..token_on_chain })
	}

	pub async fn save_chain(
		db: &DbConn,
		chain_meta: chain_meta::Model,
	) -> Result<chain_meta::Model, DbErr> {
		let active_model: chain_meta::ActiveModel = chain_meta.clone().into();
		let on_conflict = OnConflict::column(chain_meta::Column::ChainId)
			.do_nothing()
			.to_owned();
		let insert_result = ChainMeta::insert(active_model.clone())
			.on_conflict(on_conflict)
			.exec(db)
			.await;
		match insert_result {
			Ok(ret) => {
				info!("insert chain result : {:?}", ret);
			}
			Err(_) => {
				info!("the chain already exited, need to update chain !");

				let res = ChainMeta::update(active_model)
					.filter(chain_meta::Column::ChainId.eq(chain_meta.chain_id.to_owned()))
					.exec(db)
					.await
					.map(|chain| chain);
				info!("update chain result : {:?}", res);
			}
		}
		Ok(chain_meta::Model { ..chain_meta })
	}

	pub async fn save_token(
		db: &DbConn,
		token_meta: token_meta::Model,
	) -> Result<token_meta::Model, DbErr> {
		let active_model: token_meta::ActiveModel = token_meta.clone().into();
		let on_conflict = OnConflict::column(token_meta::Column::TokenId)
			.do_nothing()
			.to_owned();
		let insert_result = TokenMeta::insert(active_model.clone())
			.on_conflict(on_conflict)
			.exec(db)
			.await;
		match insert_result {
			Ok(ret) => {
				info!("insert token result : {:?}", ret);
			}
			Err(_) => {
				info!(" token already exited, need to update token !");
				let res = TokenMeta::update(active_model)
					.filter(token_meta::Column::TokenId.eq(token_meta.token_id.to_owned()))
					.exec(db)
					.await
					.map(|token| token);
				info!("update token result : {:?}", res);
			}
		}

		Ok(token_meta::Model { ..token_meta })
	}

	pub async fn save_ticket(db: &DbConn, ticket: ticket::Model) -> Result<ticket::Model, DbErr> {
		let active_model: ticket::ActiveModel = ticket.clone().into();
		let on_conflict = OnConflict::column(ticket::Column::TicketId)
			.do_nothing()
			.to_owned();
		let insert_result = Ticket::insert(active_model.clone())
			.on_conflict(on_conflict)
			.exec(db)
			.await;
		match insert_result {
			Ok(ret) => {
				info!("insert ticket result : {:?}", ret);
			}
			Err(_) => {
				info!("the ticket already exited, need to update ticket !");

				let res = Ticket::update(active_model)
					.filter(ticket::Column::TicketId.eq(ticket.ticket_id.to_owned()))
					.exec(db)
					.await
					.map(|ticket| ticket);
				info!("update ticket result : {:?}", res);
			}
		}

		Ok(ticket::Model { ..ticket })
	}

	pub async fn update_ticket_status(
		db: &DbConn,
		ticket: ticket::Model,
		status: TicketStatus,
	) -> Result<ticket::Model, DbErr> {
		let mut active_model: ticket::ActiveModel = ticket.into();
		active_model.status = Set(status.to_owned());
		let ticket = active_model.update(db).await?;
		Ok(ticket)
	}

	pub async fn update_tikcet_tx_hash(
		db: &DbConn,
		ticket: ticket::Model,
		tx_hash: TxHash,
	) -> Result<ticket::Model, DbErr> {
		let mut active_model: ticket::ActiveModel = ticket.into();
		active_model.tx_hash = Set(tx_hash.to_owned());
		let ticket = active_model.update(db).await?;
		Ok(ticket)
	}

	pub async fn update_tikcet_amount(
		db: &DbConn,
		ticket: ticket::Model,
		amount: i64,
	) -> Result<ticket::Model, DbErr> {
		let mut active_model: ticket::ActiveModel = ticket.into();
		active_model.amount = Set(amount.to_owned());
		let ticket = active_model.update(db).await?;
		Ok(ticket)
	}

	pub async fn update_tikcet_sender(
		db: &DbConn,
		ticket: ticket::Model,
		sender: String,
	) -> Result<ticket::Model, DbErr> {
		let mut active_model: ticket::ActiveModel = ticket.into();
		active_model.sender = Set(Some(sender.to_owned()));
		let ticket = active_model.update(db).await?;
		Ok(ticket)
	}
}
