use anyhow::*;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;

#[allow(non_camel_case_types)]
type numeric = i64;

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/customs/graphql/schema.json",
	query_path = "src/customs/graphql/term_amount.graphql",
	response_derives = "Debug"
)]
pub struct AmountQuery;

pub async fn query_terms_amount(variables: &str) -> Result<i64, anyhow::Error> {
	let variables: amount_query::Variables = amount_query::Variables {
		token_id: variables.to_string(),
	};
	let request_body = AmountQuery::build_query(variables);

	let client = Client::new();

	let response = client
		.post("https://runescan-hasura-mainnet-219952077564.us-central1.run.app/v1/graphql")
		.json(&request_body)
		.send()
		.await?;

	let response_body: Response<amount_query::ResponseData> = response.json().await?;

	if let Some(data) = response_body.data {
		if let Some(runes) = data.runes.first() {
			if let Some(terms_amount) = runes.terms_amount {
				return Ok(terms_amount);
			}
		}
	}

	Err(anyhow!("Failed to get terms amount from response"))
}
