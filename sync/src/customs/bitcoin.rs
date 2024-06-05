use crate::{
	types::{Ticket, TicketId, TicketStatus, TicketType, TxAction},
	with_omnity_canister,
};
use candid::{CandidType, Decode, Encode};
use ic_btc_interface::Txid;

use crate::service::{Mutation, Query};
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

#[derive(CandidType, Deserialize)]
pub struct GetGenTicketReqsArgs {
	pub start_txid: Option<Txid>,
	pub max_count: u64,
}

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
		"provided rune_id was not valid".fmt(f)
	}
}

impl Error for ParseRuneIdError {
	fn description(&self) -> &str {
		"failed to parse rune_id"
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
				"{:?} generate ticket on bitcion customs ... ",
				chrono::Utc::now()
			);
			let args: Vec<u8> = Encode!(&args)?;
			let ret = agent
				.update(&canister_id, "generate_ticket")
				.with_arg(args)
				.call_and_wait()
				.await?;
			info!(" mock generate ticket on bitcion customs ret: {:?}  ", ret);

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
				"{:?} mock finalized ticket on bitcion customs ... ",
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
				"{:?} mock finalized release token on bitcion customs ... ",
				chrono::Utc::now()
			);
			let args: Vec<u8> = Encode!(&ticket_id, &status)?;
			let ret = agent
				.update(&canister_id, "mock_finalized_release_token")
				.with_arg(args)
				.call_and_wait()
				.await?;
			let ret = Decode!(&ret, ())?;
			info!("mock finalized release token ret: {:?}  ", ret);

			Ok(())
		},
	)
	.await
}

// sync tickets that transfered from customs to routes
pub async fn sync_pending_tickets_from_bitcoin(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_BITCOIN_CANISTER_ID",
		|agent, canister_id| async move {
			info!(
				"{:?} syncing tickets from bitcoin custom ... ",
				chrono::Utc::now()
			);

			let args: Vec<u8> = Encode!(&Vec::<u8>::new())?;
			let ret = agent
				.query(&canister_id, "get_pending_gen_ticket_size")
				.with_arg(args)
				.call()
				.await?;
			let ticket_size = Decode!(&ret, u64)?;
			info!("pending ticket size: {:?}", ticket_size);

			let mut offset = 0u64;
			let limit = FETCH_LIMIT;
			while offset < ticket_size {
				let args = Encode!(&offset, &limit)?;
				let ret = agent
					.query(&canister_id, "get_pending_gen_tickets")
					.with_arg(args)
					.call()
					.await?;
				let pending_tickets: Vec<GenTicketRequest> = Decode!(&ret, Vec<GenTicketRequest>)?;
				info!(
					"need to sync pending tickets {}: {:?}",
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
	with_omnity_canister("OMNITY_CUSTOMS_BITCOIN_CANISTER_ID", |agent, canister_id| async move {
		info!(
			"{:?} syncing release token status from bitcoin ... ",
			chrono::Utc::now()
		);
		//step1: get ticket that dest is bitcion and status is waiting for comformation by dst
		let unconfirmed_tickets = Query::get_unconfirmed_tickets(db, CUSTOMS_CHAIN_ID.to_owned()).await?;

		//step2: get release_token_status by ticket id
		for unconfirmed_ticket in unconfirmed_tickets {
			info!("unconfirmed ticket({:?}) ", unconfirmed_ticket);
			let args = Encode!(&unconfirmed_ticket.ticket_id)?;
			let ret = agent
				.query(&canister_id, "release_token_status")
				.with_arg(args)
				.call()
				.await?;
			let mint_token_status: ReleaseTokenStatus = Decode!(&ret, ReleaseTokenStatus)?;
			if matches!(mint_token_status, ReleaseTokenStatus::Confirmed(ref s) if s.eq(&unconfirmed_ticket.ticket_id))
			{
				//step3: update ticket status to finalized
				let ticket_modle = Mutation::update_ticket_status(
					db,
					unconfirmed_ticket,
					crate::entity::sea_orm_active_enums::TicketStatus::Finalized,
				)
				.await?;
				info!(
					"ticket id({:?}) finally status:{:?} ",
					ticket_modle.ticket_id, ticket_modle.status
				);
			} else {
				info!(
					"ticket id({:?}) current status {:?}",
					unconfirmed_ticket.ticket_id, mint_token_status
				);
			}
		}

		Ok(())
	})
	.await
}
