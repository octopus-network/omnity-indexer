use crate::graphql::terms_amount::query_terms_amount;
use crate::service::{Delete, Mutation, Query};
use crate::{
	types::{Ticket, TicketId, TicketStatus, TicketType, TxAction},
	with_omnity_canister, Arg,
};
use candid::{CandidType, Decode, Encode};
use ic_btc_interface::Txid;
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::{
	error::Error,
	fmt::{self, Display, Formatter},
	str::FromStr,
};

pub const CUSTOMS_CHAIN_ID: &str = "Bitcoin";
const FETCH_LIMIT: u64 = 50;

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenTicketRequest {
	pub address: String,
	pub target_chain_id: String,
	pub receiver: String,
	pub token_id: String,
	pub rune_id: RuneId,
	pub amount: u128,
	pub txid: Txid,
	pub received_at: u64,
}

#[derive(
	candid::CandidType,
	Clone,
	Debug,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Copy,
	Default,
	Serialize,
	Deserialize,
)]
pub struct RuneId {
	pub block: u64,
	pub tx: u32,
}

impl Display for RuneId {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}:{}", self.block, self.tx,)
	}
}

impl FromStr for RuneId {
	type Err = ParseRuneIdError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (height, index) = s.split_once(':').ok_or_else(|| ParseRuneIdError)?;

		Ok(Self {
			block: height.parse().map_err(|_| ParseRuneIdError)?,
			tx: index.parse().map_err(|_| ParseRuneIdError)?,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRuneIdError;

impl fmt::Display for ParseRuneIdError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		"Provided rune_id was not valid".fmt(f)
	}
}

impl Error for ParseRuneIdError {
	fn description(&self) -> &str {
		"Failed to parse rune_id"
	}
}

/// The status of a release_token request.
#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum ReleaseTokenStatus {
	/// The custom has no data for this request.
	/// The request id is either invalid or too old.
	Unknown,
	/// The request is in the batch queue.
	Pending,
	/// Waiting for a signature on a transaction satisfy this request.
	Signing,
	/// Sending the transaction satisfying this request.
	Sending(String),
	/// Awaiting for confirmations on the transaction satisfying this request.
	Submitted(String),
	/// Confirmed a transaction satisfying this request.
	Confirmed(String),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketArgs {
	pub target_chain_id: String,
	pub receiver: String,
	pub rune_id: String,
	pub amount: u128,
	pub txid: String,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalizedStatus {
	/// The transaction that release token got enough confirmations.
	Confirmed(Txid),
}

// mock: generate ticket from customs
pub async fn gen_bitcoin_ticket(args: GenerateTicketArgs) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_BITCOIN_CANISTER_ID",
		|agent, canister_id| async move {
			info!(
				"{:?} Generate ticket on bitcion customs ... ",
				chrono::Utc::now()
			);
			let args: Vec<u8> = Encode!(&args)?;
			let ret = agent
				.update(&canister_id, "generate_ticket")
				.with_arg(args)
				.call_and_wait()
				.await?;
			info!(" Mock generate ticket on bitcion customs ret: {:?}  ", ret);

			Ok(())
		},
	)
	.await
}

// mock: finalizd release token
pub async fn mock_finalized_ticket(ticket_id: TicketId) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_BITCOIN_CANISTER_ID",
		|agent, canister_id| async move {
			info!(
				"{:?} Mock finalized ticket on bitcion customs ... ",
				chrono::Utc::now()
			);
			let args: Vec<u8> = Encode!(&ticket_id)?;
			agent
				.update(&canister_id, "mock_finalized_ticket")
				.with_arg(args)
				.call()
				.await?;

			Ok(())
		},
	)
	.await
}
// mock: finalizd release token
pub async fn mock_finalized_release_token(
	ticket_id: TicketId,
	status: FinalizedStatus,
) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_BITCOIN_CANISTER_ID",
		|agent, canister_id| async move {
			info!(
				"{:?} Mock finalized release token on bitcion customs ... ",
				chrono::Utc::now()
			);
			let args: Vec<u8> = Encode!(&ticket_id, &status)?;
			let ret = agent
				.update(&canister_id, "mock_finalized_release_token")
				.with_arg(args)
				.call_and_wait()
				.await?;
			let ret = Decode!(&ret, ())?;
			info!("Mock finalized release token ret: {:?}  ", ret);

			Ok(())
		},
	)
	.await
}

// mock: sync tickets that transfered from customs to routes
pub async fn sync_pending_tickets_from_bitcoin(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_BITCOIN_CANISTER_ID",
		|agent, canister_id| async move {
			let ticket_size = Arg::query_method(
				Arg::V(Vec::<u8>::new()),
				agent.clone(),
				canister_id,
				"get_pending_gen_ticket_size",
				"Syncing tickets from bitcoin custom ...",
				"Pending ticket size: ",
				None,
				None,
				"u64",
			)
			.await?
			.convert_to_u64();

			let mut offset = 0u64;
			let limit = FETCH_LIMIT;
			while offset < ticket_size {
				let pending_tickets = Arg::U(offset)
					.query_method(
						agent.clone(),
						canister_id,
						"get_pending_gen_tickets",
						" ",
						" ",
						Some(limit),
						None,
						"Vec<GenTicketRequest>",
					)
					.await?
					.convert_to_vec_gen_ticket_request();

				info!(
					"Need to sync pending tickets {}: {:?}",
					offset, pending_tickets
				);
				for pending_ticket in pending_tickets.iter() {
					let ticket_modle = Ticket::new(
						pending_ticket.txid.to_string(),
						None,
						TicketType::Normal,
						pending_ticket.received_at,
						CUSTOMS_CHAIN_ID.to_owned(),
						pending_ticket.target_chain_id.to_owned(),
						TxAction::Transfer,
						pending_ticket.token_id.to_string(),
						pending_ticket.amount.to_string(),
						None,
						pending_ticket.receiver.to_owned(),
						None,
						TicketStatus::WaitingForConfirmByDest,
						" ".to_string(),
					)
					.into();
					Mutation::save_ticket(db, ticket_modle).await?;
				}
				offset += pending_tickets.len() as u64;
				if pending_tickets.is_empty() {
					break;
				}
			}
			Ok(())
		},
	)
	.await
}

// sync tickets status that transfered from routes to customs
pub async fn sync_ticket_status_from_bitcoin(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_BITCOIN_CANISTER_ID",
		|agent, canister_id| async move {
			info!(
				"{:?} Syncing release token status from bitcoin ... ",
				chrono::Utc::now()
			);
			//step1: get ticket that dest is bitcion and status is waiting for comformation by dst
			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, CUSTOMS_CHAIN_ID.to_owned()).await?;

			//step2: get release_token_status by ticket id
			for unconfirmed_ticket in unconfirmed_tickets {
				let mint_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"release_token_status",
						"Unconfirmed ticket: ",
						"Mint token status result: ",
						None,
						None,
						"ReleaseTokenStatus",
					)
					.await?
					.convert_to_release_token_status();

				if let ReleaseTokenStatus::Confirmed(tx_hash) = mint_token_status {
					//step3: update ticket status to finalized
					let ticket_model = Mutation::update_ticket_status_n_txhash(
						db,
						unconfirmed_ticket.clone(),
						crate::entity::sea_orm_active_enums::TicketStatus::Finalized,
						tx_hash,
					)
					.await?;

					info!(
						"Ticket id({:?}) finally status:{:?} and its ICP hash is {:?} ",
						ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
					);
				} else {
					info!(
						"Ticket id({:?}) current status {:?}",
						unconfirmed_ticket.ticket_id, mint_token_status
					);
				}
			}
			Ok(())
		},
	)
	.await
}

// update mint tickets meta
pub async fn update_mint_tickets(db: &DbConn) -> Result<(), Box<dyn Error>> {
	// Find all the mint tickets
	let non_updated_mint_tickets = Query::get_non_updated_mint_tickets(db).await?;

	for ticket in non_updated_mint_tickets {
		if let Some(token_id) = ticket.token.as_str().strip_prefix("Bitcoin-runes-") {
			// Fetch the amount from the runescan graphql api
			let amount = query_terms_amount(token_id).await.unwrap();
			// Insert the amount into the ticket meta
			let updated_ticket = Mutation::update_tikcet_amount(db, ticket, amount).await?;
			info!(
				"Ticket id({:?}) has changed its amount to {:?}",
				updated_ticket.ticket_id, updated_ticket.amount
			);
		}
	}

	let updated_mint_tickets = Query::get_updated_mint_tickets(db).await?;
	for mint_ticket in updated_mint_tickets {
		// Retrieval the tx_hash from each mint tickets
		if let Some(ticket_should_be_removed) =
			Query::get_ticket_by_id(db, mint_ticket.clone().tx_hash).await?
		{
			// Save the ticket that contains the tx_hash as the ticket_id to DeletedMintTicket
			let _ = Mutation::save_deleted_mint_ticket(db, ticket_should_be_removed.clone().into()).await?;

			// Remove the ticket that contains the tx_hash as the ticket_id
			let row =
				Delete::remove_ticket_by_id(db, ticket_should_be_removed.clone().ticket_id).await?;
			info!(
				"Ticket id({:?}) has been removed and {:?} row has been deleted",
				ticket_should_be_removed.clone().ticket_id,
				row
			);
		}
	}
	Ok(())
}
