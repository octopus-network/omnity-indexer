use crate::service::{Mutation, Query};
use crate::{with_omnity_canister, Arg};
use log::info;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const ICP_CUSTOM_CHAIN_ID: &str = "sICP";

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ICPCustomRelaseTokenStatus {
	Finalized { tx_hash: String },
	Unknown,
}

// sync tickets status that transfered from routes to icp custom
pub async fn sync_ticket_status_from_sicp(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			info!("Syncing release token status from icp custom ... ");

			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, ICP_CUSTOM_CHAIN_ID.to_owned()).await?;

			for unconfirmed_ticket in unconfirmed_tickets {
				let release_icp_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"mint_token_status",
						"Unconfirmed ICP custom ticket: ",
						"Release ICP token status result: ",
						None,
						None,
						"ICPCustomRelaseTokenStatus",
					)
					.await?
					.convert_to_release_icp_token_status();

				if let ICPCustomRelaseTokenStatus::Finalized { tx_hash } = release_icp_token_status
				{
					let ticket_model = Mutation::update_ticket(
						db,
						unconfirmed_ticket.clone(),
						Some(crate::entity::sea_orm_active_enums::TicketStatus::Finalized),
						Some(Some(tx_hash)),
						None,
						None,
						None,
						None,
					)
					.await?;

					info!(
						"Ticket id({:?}) finally status:{:?} and its ICP custom hash is {:?} ",
						ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
					);
				}
				// match release_icp_token_status {
				// 	ICPCustomRelaseTokenStatus::Finalized { tx_hash } => {
				// 		let ticket_model = Mutation::update_ticket(
				// 			db,
				// 			unconfirmed_ticket.clone(),
				// 			Some(crate::entity::sea_orm_active_enums::TicketStatus::Finalized),
				// 			Some(Some(tx_hash)),
				// 			None,
				// 			None,
				// 			None,
				// 			None,
				// 		)
				// 		.await?;

				// 		info!(
				// 			"Ticket id({:?}) finally status:{:?} and its ICP custom hash is {:?} ",
				// 			ticket_model.ticket_id, ticket_model.status, ticket_model.tx_hash
				// 		);
				// 	}

				// 	ICPCustomRelaseTokenStatus::Unknown => {
				// 		info!(
				// 			"Ticket id({:?}) current status {:?}",
				// 			unconfirmed_ticket.ticket_id, release_icp_token_status
				// 		);
				// 	}
				// }
			}

			Ok(())
		},
	)
	.await
}
