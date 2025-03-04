use anyhow::*;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;

#[allow(non_camel_case_types)]
type json = serde_json::Value;

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/customs/graphql/schema.json",
	query_path = "src/customs/graphql/sender.graphql",
	response_derives = "Debug"
)]
pub struct SenderQuery;

pub async fn query_sender_fm_runescan(address: String) -> Result<String, anyhow::Error> {
	let variables: sender_query::Variables = sender_query::Variables { address: address };
	let request_body = SenderQuery::build_query(variables);
	let client = Client::new();
	let response = client
		.post("not being used")
		.json(&request_body)
		.send()
		.await
		.expect("Error sending request");

	let response_body: Response<sender_query::ResponseData> =
		response.json().await.expect("Error deserializing response");
	if let Some(data) = response_body.data {
		Ok(data.transactions[0].transaction["inputs"][0]["address"].to_string())
	} else {
		Err(format_err!("Missing response data"))
	}
}

pub async fn query_sender_fm_mempool(ticket_id: &str) -> Result<String, anyhow::Error> {
	let client = reqwest::Client::new();
	let url = "https://mempool.space/api/tx/".to_string() + ticket_id;
	let response = client.get(url).send().await?;

	let body = response.text().await?;
	let mut a = serde_json::from_str::<serde_json::Value>(&body).unwrap();

	if let Some(vin) = a.get_mut("vin") {
		match &vin[0]["prevout"]["scriptpubkey_address"].as_str() {
			Some(add) => Ok(add.to_string()),
			None => Err(format_err!("Missing address data")),
		}
	} else {
		Err(format_err!("Missing response data"))
	}
}
