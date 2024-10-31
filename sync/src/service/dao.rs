use crate::entity::sea_orm_active_enums::{TicketStatus, TxAction};
use crate::entity::{
	chain_meta, deleted_mint_ticket, pending_ticket, ticket, token_ledger_id_on_chain, token_meta,
	token_on_chain, token_volume,
};
use crate::entity::{
	chain_meta::Entity as ChainMeta, deleted_mint_ticket::Entity as DeletedMintTicket,
	pending_ticket::Entity as PendingTicket, ticket::Entity as Ticket,
	token_ledger_id_on_chain::Entity as TokenLedgerIdOnChain, token_meta::Entity as TokenMeta,
	token_on_chain::Entity as TokenOnChain, token_volume::Entity as TokenVolume,
};
use log::info;
use sea_orm::{sea_query::OnConflict, *};

pub struct Query;

impl Query {
	pub async fn get_all_tokens(db: &DbConn) -> Result<Vec<token_meta::Model>, DbErr> {
		TokenMeta::find().all(db).await
	}
	pub async fn get_ticket_by_id(
		db: &DbConn,
		ticket_id: String,
	) -> Result<Option<ticket::Model>, DbErr> {
		Ticket::find_by_id(ticket_id).one(db).await
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
	// pub async fn get_latest_pending_ticket(
	// 	db: &DbConn,
	// ) -> Result<Option<pending_ticket::Model>, DbErr> {
	// 	PendingTicket::find()
	// 		.filter(pending_ticket::Column::TicketIndex.is_not_null())
	// 		.order_by_desc(pending_ticket::Column::TicketIndex)
	// 		.one(db)
	// 		.await
	// }
	pub async fn get_unconfirmed_tickets(
		db: &DbConn,
		dest: String,
	) -> Result<Vec<ticket::Model>, DbErr> {
		Ticket::find()
			.filter(
				Condition::all()
					// The ticket is not finalized
					.add(ticket::Column::Status.ne(TicketStatus::Finalized))
					.add(ticket::Column::Status.ne(TicketStatus::Unknown))
					// The ticket's destination chain matches `dest`
					.add(ticket::Column::DstChain.eq(dest)),
			)
			.all(db)
			.await
	}

	pub async fn get_unconfirmed_deleted_mint_tickets(
		db: &DbConn,
		dest: String,
	) -> Result<Vec<deleted_mint_ticket::Model>, DbErr> {
		DeletedMintTicket::find()
			.filter(
				Condition::all()
					.add(deleted_mint_ticket::Column::Status.ne(TicketStatus::Finalized))
					.add(deleted_mint_ticket::Column::DstChain.eq(dest)),
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
					.add(ticket::Column::Amount.eq(0.to_string())),
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
					.add(ticket::Column::Amount.ne(0.to_string()))
					.add(
						Condition::any()
							.add(ticket::Column::IntermediateTxHash.is_null())
							.add(ticket::Column::TxHash.is_null()),
					),
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

	pub async fn get_token_tickets(
		db: &DbConn,
		token: String,
	) -> Result<Vec<ticket::Model>, DbErr> {
		Ticket::find()
			.filter(Condition::all().add(ticket::Column::Token.eq(token)))
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

	pub async fn remove_chains(db: &DbConn) -> Result<DeleteResult, DbErr> {
		ChainMeta::delete_many()
			.filter(Condition::all().add(chain_meta::Column::ChainId.is_not_null()))
			.exec(db)
			.await
	}

	pub async fn remove_tokens(db: &DbConn) -> Result<DeleteResult, DbErr> {
		TokenMeta::delete_many()
			.filter(Condition::all().add(token_meta::Column::TokenId.is_not_null()))
			.exec(db)
			.await
	}

	pub async fn remove_token_on_chains(db: &DbConn) -> Result<DeleteResult, DbErr> {
		TokenOnChain::delete_many()
			.filter(Condition::all().add(token_on_chain::Column::ChainId.is_not_null()))
			.exec(db)
			.await
	}

	pub async fn remove_token_ledger_id_on_chain(db: &DbConn) -> Result<DeleteResult, DbErr> {
		TokenLedgerIdOnChain::delete_many()
			.filter(Condition::all().add(token_ledger_id_on_chain::Column::ChainId.is_not_null()))
			.exec(db)
			.await
	}

	pub async fn remove_tickets(db: &DbConn) -> Result<DeleteResult, DbErr> {
		Ticket::delete_many()
			.filter(Condition::all().add(ticket::Column::TicketId.is_not_null()))
			.exec(db)
			.await
	}

	pub async fn remove_deleted_mint_tickets(db: &DbConn) -> Result<DeleteResult, DbErr> {
		DeletedMintTicket::delete_many()
			.filter(Condition::all().add(deleted_mint_ticket::Column::TicketId.is_not_null()))
			.exec(db)
			.await
	}

	pub async fn remove_pending_mint_tickets(db: &DbConn) -> Result<DeleteResult, DbErr> {
		PendingTicket::delete_many()
			.filter(Condition::all().add(pending_ticket::Column::TicketIndex.is_not_null()))
			.exec(db)
			.await
	}

	pub async fn remove_token_volumes(db: &DbConn) -> Result<DeleteResult, DbErr> {
		TokenVolume::delete_many()
			.filter(Condition::all().add(token_volume::Column::TokenId.is_not_null()))
			.exec(db)
			.await
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
				let _res = TokenLedgerIdOnChain::update(active_model)
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
				info!("the token ledger id already exists, updated it !");
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
				let _res = TokenOnChain::update(active_model)
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
				info!("the token on chain already exists, updated it !");
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
				let _res = ChainMeta::update(active_model)
					.filter(chain_meta::Column::ChainId.eq(chain_meta.chain_id.to_owned()))
					.exec(db)
					.await
					.map(|chain| chain);
				info!("the chain already exists, updated chain !");
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
				let _res = TokenMeta::update(active_model)
					.filter(token_meta::Column::TokenId.eq(token_meta.token_id.to_owned()))
					.exec(db)
					.await
					.map(|token| token);
				info!("token already exists, updated token !");
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
				if let Some(t) = Query::get_ticket_by_id(db, ticket.clone().ticket_id).await? {
					if t.ticket_seq == None && t.status == TicketStatus::Finalized {
						let model = Self::update_ticket(
							db,
							ticket.clone(),
							None,
							None,
							None,
							None,
							None,
							Some(ticket.clone().ticket_seq),
						)
						.await?;
						info!("update ticket seq result {:?}", model.ticket_seq);
					}
				}
			}
		}

		Ok(ticket::Model { ..ticket })
	}

	pub async fn save_deleted_mint_ticket(
		db: &DbConn,
		deleted_ticket: deleted_mint_ticket::Model,
	) -> Result<deleted_mint_ticket::Model, DbErr> {
		let active_model: deleted_mint_ticket::ActiveModel = deleted_ticket.clone().into();
		let on_conflict = OnConflict::column(deleted_mint_ticket::Column::TicketId)
			.do_nothing()
			.to_owned();
		let insert_result = DeletedMintTicket::insert(active_model.clone())
			.on_conflict(on_conflict)
			.exec(db)
			.await;
		match insert_result {
			Ok(ret) => {
				info!("insert deleted mint ticket result : {:?}", ret);
			}
			Err(_) => {
				info!("the deleted mint ticket already exists");
			}
		}

		Ok(deleted_mint_ticket::Model { ..deleted_ticket })
	}

	// pub async fn save_pending_ticket(
	// 	db: &DbConn,
	// 	pending_ticket: pending_ticket::Model,
	// ) -> Result<pending_ticket::Model, DbErr> {
	// 	let active_model: pending_ticket::ActiveModel = pending_ticket.clone().into();
	// 	let on_conflict = OnConflict::column(pending_ticket::Column::TicketId)
	// 		.do_nothing()
	// 		.to_owned();
	// 	let insert_result = PendingTicket::insert(active_model.clone())
	// 		.on_conflict(on_conflict)
	// 		.exec(db)
	// 		.await;
	// 	match insert_result {
	// 		Ok(ret) => {
	// 			info!("insert pending ticket result : {:?}", ret);
	// 		}
	// 		Err(_) => {
	// 			info!("the pending ticket already exists, need to update ticket !");
	// 			let res = PendingTicket::update(active_model)
	// 				.filter(
	// 					pending_ticket::Column::TicketId.eq(&pending_ticket.ticket_id.to_owned()),
	// 				)
	// 				.exec(db)
	// 				.await
	// 				.map(|ticket| ticket);
	// 			info!("update pending ticket result : {:?}", res);
	// 		}
	// 	}
	// 	Ok(pending_ticket::Model { ..pending_ticket })
	// }
	pub async fn save_pending_ticket_index(
		db: &DbConn,
		pending_ticket: pending_ticket::Model,
	) -> Result<pending_ticket::Model, DbErr> {
		let active_model: pending_ticket::ActiveModel = pending_ticket.clone().into();
		let on_conflict = OnConflict::column(pending_ticket::Column::TicketIndex)
			.do_nothing()
			.to_owned();
		let insert_result = PendingTicket::insert(active_model.clone())
			.on_conflict(on_conflict)
			.exec(db)
			.await;
		match insert_result {
			Ok(ret) => {
				info!("insert pending ticket index result : {:?}", ret);
			}
			Err(_) => {
				let _res = PendingTicket::update(active_model)
					.filter(
						pending_ticket::Column::TicketIndex
							.eq(pending_ticket.ticket_index.to_owned()),
					)
					.exec(db)
					.await
					.map(|ticket| ticket);
				info!("the pending ticket index already exists, updated ticket!",);
			}
		}

		Ok(pending_ticket::Model { ..pending_ticket })
	}

	pub async fn save_token_volume(
		db: &DbConn,
		token_volume: token_volume::Model,
	) -> Result<token_volume::Model, DbErr> {
		let active_model: token_volume::ActiveModel = token_volume.clone().into();
		let on_conflict = OnConflict::column(token_volume::Column::TokenId)
			.do_nothing()
			.to_owned();
		let insert_result = TokenVolume::insert(active_model.clone())
			.on_conflict(on_conflict)
			.exec(db)
			.await;
		match insert_result {
			Ok(ret) => {
				info!("insert token volume result : {:?}", ret);
			}
			Err(_) => {
				let model = Self::update_token_volume(
					db,
					token_volume.clone(),
					token_volume.clone().ticket_count,
					token_volume.clone().historical_volume,
				)
				.await?;
				info!("the token volume already exists, updated it ! {:?}", model);
			}
		}
		Ok(token_volume::Model { ..token_volume })
	}

	pub async fn update_ticket(
		db: &DbConn,
		ticket: ticket::Model,
		status: Option<TicketStatus>,
		tx_hash: Option<Option<String>>,
		amount: Option<String>,
		sender: Option<Option<String>>,
		intermediate_tx_hash: Option<Option<String>>,
		seq: Option<Option<i64>>,
	) -> Result<ticket::Model, DbErr> {
		let mut active_model: ticket::ActiveModel = ticket.into();
		if let Some(_status) = status {
			active_model.status = Set(_status);
		}
		if let Some(_tx_hash) = tx_hash {
			active_model.tx_hash = Set(_tx_hash);
		}
		if let Some(_amount) = amount {
			active_model.amount = Set(_amount);
		}
		if let Some(_sender) = sender {
			active_model.sender = Set(_sender);
		}
		if let Some(_intermediate_tx_hash) = intermediate_tx_hash {
			active_model.intermediate_tx_hash =
				Set(Some(_intermediate_tx_hash.to_owned().expect("no hash")));
		}
		if let Some(_seq) = seq {
			active_model.ticket_seq = Set(Some(_seq.to_owned().expect("no seq")));
		}
		let ticket = active_model.update(db).await?;
		Ok(ticket)
	}

	pub async fn update_ticket_tx_hash(
		db: &DbConn,
		ticket: ticket::Model,
		tx_hash: Option<String>,
	) -> Result<ticket::Model, DbErr> {
		let mut active_model: ticket::ActiveModel = ticket.into();
		active_model.tx_hash = Set(tx_hash);
		let ticket = active_model.update(db).await?;
		Ok(ticket)
	}

	pub async fn update_token_volume(
		db: &DbConn,
		token_volume: token_volume::Model,
		len: String,
		volume: String,
	) -> Result<token_volume::Model, DbErr> {
		let mut active_model: token_volume::ActiveModel = token_volume.into();
		active_model.ticket_count = Set(len);
		active_model.historical_volume = Set(volume);
		let token_volume = active_model.update(db).await?;
		Ok(token_volume)
	}
}
