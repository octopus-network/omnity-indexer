use crate::graphql::terms_amount::query_terms_amount;
use crate::service::{Delete, Mutation, Query};
use crate::{with_omnity_canister, Arg};
use log::info;
use sea_orm::DbConn;
use serde::Deserialize;
use std::error::Error;

pub const BTC_CUSTOM_CHAIN_ID: &str = "Bitcoin";

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

// sync tickets status that transfered from routes to BTC custom
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
				Query::get_unconfirmed_tickets(db, BTC_CUSTOM_CHAIN_ID.to_owned()).await?;

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

				if let ReleaseTokenStatus::Submitted(tx_hash)
				| ReleaseTokenStatus::Confirmed(tx_hash) = mint_token_status
				{
					//step3: update ticket status to finalized
					let ticket_model = Mutation::update_ticket_status_n_txhash(
						db,
						unconfirmed_ticket.clone(),
						crate::entity::sea_orm_active_enums::TicketStatus::Finalized,
						Some(tx_hash),
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
			let updated_ticket = Mutation::update_tikcet_amount(db, ticket, amount.to_string()).await?;
			info!(
				"Ticket id({:?}) has changed its amount to {:?}",
				updated_ticket.ticket_id, updated_ticket.amount
			);
		}
	}
	Ok(())
}

// update deleted mint tickets meta
pub async fn update_deleted_mint_tickets(db: &DbConn) -> Result<(), Box<dyn Error>> {
	let updated_mint_tickets = Query::get_updated_mint_tickets(db).await?;
	for mint_ticket in updated_mint_tickets {
		match Query::get_ticket_by_id(db, mint_ticket.clone().tx_hash.unwrap()).await? {
			Some(ticket_should_be_removed) => {
				// Retrieval the tx_hash from each mint tickets
				// update the ticket_should_be_removed status, then move the mint ticket tx_hash to
				// the intermedieate tx_hash and then the tx_hash to mint ticket tx_hash
				match ticket_should_be_removed.clone().tx_hash {
					Some(tx_hash) => {
						// fetch the tx_hash from the mint ticket and put it in intermediate_tx_hash
						let intermediate_tx_hash = mint_ticket.clone().tx_hash;
						let _ = Mutation::update_ticket_intermediate_tx_hash(
							db,
							mint_ticket.clone(),
							intermediate_tx_hash,
						)
						.await?;
						// put the hash to mint ticket tx_hash
						let _ =
							Mutation::update_ticket_tx_hash(db, mint_ticket, Some(tx_hash.clone()))
								.await?;

						// Save the ticket that contains the tx_hash as the ticket_id to
						// DeletedMintTicket
						let _ = Mutation::save_deleted_mint_ticket(
							db,
							ticket_should_be_removed.clone().into(),
						)
						.await?;

						// Update sender/seq only if they are needed

						// Remove the ticket that contains the tx_hash as the ticket_id
						let row = Delete::remove_ticket_by_id(
							db,
							ticket_should_be_removed.clone().ticket_id,
						)
						.await?;
						info!(
							"Ticket id({:?}) has been removed and {:?} row has been deleted",
							ticket_should_be_removed.clone().ticket_id,
							row
						);
					}
					None => {
						let intermediate_tx_hash = mint_ticket.clone().tx_hash;
						let _ = Mutation::update_ticket_intermediate_tx_hash(
							db,
							mint_ticket.clone(),
							intermediate_tx_hash,
						)
						.await?;
						let _ = Mutation::update_ticket_tx_hash(db, mint_ticket, None).await?;
					}
				}
			}
			None => {
				//update mint tickets status if there is no corresponding transfer tickets.
				let _ = Mutation::update_ticket_status(
					db,
					mint_ticket.clone(),
					crate::entity::sea_orm_active_enums::TicketStatus::Unknown,
				)
				.await?;
			}
		}
	}
	Ok(())
}
