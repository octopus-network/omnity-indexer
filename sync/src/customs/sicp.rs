use crate::service::{Mutation, Query};
use crate::{token_ledger_id_on_chain, with_omnity_canister, Arg};
use log::info;
use reqwest::Client;
use sea_orm::DbConn;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub const ICP_CUSTOM_CHAIN_ID: &str = "sICP";

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ICPCustomRelaseTokenStatus {
	Finalized { tx_hash: String },
	Unknown,
}

async fn fetch_transactions(block_height: &str) -> Result<String, Box<dyn Error>> {
	let client = Client::new();
	let params = [("sort_by", "block_height"), ("block_height", block_height)];
	let response = client
		.get("https://ledger-api.internetcomputer.org/transactions")
		.query(&params)
		.send()
		.await?;

	if let true = response.status().is_success() {
		let body = response.text().await?;
		if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) {
			if let Some(layer_one) = value.as_object() {
				if let Some(layer_two) = layer_one.get("blocks") {
					if let Some(layer_there) = layer_two[0].as_object() {
						if let Some(transaction_hash) = layer_there.get("transaction_hash") {
							let mut updated_hash = transaction_hash.to_string();
							updated_hash.replace_range(0..1, "");
							updated_hash.replace_range((updated_hash.len() - 1).., "");
							return Ok(updated_hash.to_string());
						} else {
							return Err("fetch icp tx error5".into());
						}
					} else {
						return Err("fetch icp tx error4".into());
					}
				} else {
					return Err("fetch icp tx error3".into());
				}
			} else {
				return Err("fetch icp tx error2".into());
			}
		} else {
			return Err("fetch icp tx error1".into());
		}
	} else {
		return Err("fetch icp tx error1".into());
	}
}

// sync tickets status that transfered from routes to icp custom
pub async fn sync_ticket_status_from_sicp(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			info!("icp custom状态更新在工作 ... ");
			let unconfirmed_tickets =
				Query::get_unconfirmed_tickets(db, ICP_CUSTOM_CHAIN_ID.to_owned()).await?;

			for unconfirmed_ticket in unconfirmed_tickets {
				let release_icp_token_status = Arg::TI(unconfirmed_ticket.ticket_id.clone())
					.query_method(
						agent.clone(),
						canister_id,
						"mint_token_status",
						None,
						None,
						"ICPCustomRelaseTokenStatus",
					)
					.await?
					.convert_to_release_icp_token_status();

				if let ICPCustomRelaseTokenStatus::Finalized { tx_hash } = release_icp_token_status
				{
					let token_id = unconfirmed_ticket.clone().token;
					let mut updated_tx_hash = String::new();
					match token_id == "sICP-native-ICP" {
						true => {
							if let Ok(icp_hash) = fetch_transactions(&tx_hash).await {
								updated_tx_hash.push_str(&icp_hash);
							}
						}
						false => {
							if let Some(rep) = Query::get_token_ledger_id_on_chain_by_id(
								db,
								ICP_CUSTOM_CHAIN_ID.to_owned(),
								token_id,
							)
							.await?
							{
								updated_tx_hash.push_str(&(rep.contract_id + "_" + &tx_hash));
							}
						}
					}
					if let Ok(ticket_model) = Mutation::update_ticket(
						db,
						unconfirmed_ticket.clone(),
						Some(crate::entity::sea_orm_active_enums::TicketStatus::Finalized),
						Some(Some(updated_tx_hash)),
						None,
						None,
						None,
						None,
					)
					.await
					{
						info!(
							"icp custom ticket id({:?}) and its hash is {:?} ",
							ticket_model.ticket_id, ticket_model.tx_hash
						);
					}
				}
			}

			Ok(())
		},
	)
	.await
}

pub async fn sync_all_icrc_token_canister_id_from_sicp(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_omnity_canister(
		"OMNITY_CUSTOMS_ICP_CANISTER_ID",
		|agent, canister_id| async move {
			info!("token canister id from sicp状态更新在工作 ... ");
			let token_canisters = Arg::V(Vec::<u8>::new())
				.query_method(
					agent.clone(),
					canister_id,
					"get_token_list",
					None,
					None,
					"Vec<Token>",
				)
				.await?
				.convert_to_vec_token();
			for token in token_canisters {
				if let Some(canister) = token.metadata.get("ledger_id") {
					let token_canister_id_on_chain_model = token_ledger_id_on_chain::Model::new(
						ICP_CUSTOM_CHAIN_ID.to_string(),
						token.token_id,
						canister.to_owned(),
					);

					let _token_canister_id_on_chain = Mutation::save_all_token_ledger_id_on_chain(
						db,
						token_canister_id_on_chain_model,
					)
					.await?;
				}
			}
			Ok(())
		},
	)
	.await
}
