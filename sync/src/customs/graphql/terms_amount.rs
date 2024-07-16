use anyhow::*;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;

#[allow(non_camel_case_types)]
type numeric = i64;

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/customs/graphql/schema.json",
	query_path = "src/customs/graphql/customs/term_amount.graphql",
	response_derives = "Debug"
)]
pub struct MyQuery;

pub async fn query_terms_amount(variables: &str) -> Result<i64, anyhow::Error> {
	let variables: my_query::Variables = my_query::Variables {
		token_id: variables.to_string(),
	};
	let request_body = MyQuery::build_query(variables);

	let client = Client::new();

	let response = client
		.post("https://hasura-secondary-graphql-engine-2252klcbva-uc.a.run.app/v1/graphql")
		.header("x-hasura-admin-secret", "C26sQVLsq3Adzd7CoHfv")
		.json(&request_body)
		.send()
		.await
		.expect("Error sending request");

	let response_body: Response<my_query::ResponseData> =
		response.json().await.expect("Error deserializing response");

	let data = &response_body.data.as_ref().expect("Missing response data");

	match &response_body
		.data
		.as_ref()
		.expect("Missing response data")
		.runes
		.len()
	{
		1 => {
			if let Some(rune_stats) = &data.runes[0].rs_rune_stats {
				if let Some(rune_stats_runes) = &rune_stats.runes {
					if let Some(amount) = &rune_stats_runes.terms_amount {
						// println!("{:?}", amount);
						Ok(*amount)
					} else {
						Err(format_err!("Missing terms_amount"))
					}
				} else {
					Err(format_err!("Missing runes"))
				}
			} else {
				Err(format_err!("Missing rs_rune_stats"))
			}
		}
		_ => Err(format_err!("Missing Rune data")),
	}
}
