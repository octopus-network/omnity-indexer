use reqwest::Client;
use graphql_client::{GraphQLQuery, Response};

#[allow(non_camel_case_types)]
type numeric = i64;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.json",
    query_path = "src/term_amount.graphql",
    response_derives = "Debug",
)]
pub struct MyQuery;

async fn query_terms_amount() -> Result<(), Box<dyn std::error::Error>> {
	let variables: my_query::Variables = my_query::Variables{token_id: "YOLO•TRADE•ZONE". to_string()};
	let request_body = MyQuery::build_query(variables);
	let client = Client::new();
	let response = client
		.post("https://hasura-secondary-graphql-engine-2252klcbva-uc.a.run.app/v1/graphql")
		.header("x-hasura-admin-secret", "C26sQVLsq3Adzd7CoHfv")
		.json(&request_body)
		.send()
		.await.expect("Error sending request");

	let response_body: Response<my_query::ResponseData> = response.json().await.expect("Error deserializing response");
	let data = &response_body.data.as_ref().expect("Missing response data");

	match &response_body.data.as_ref().expect("Missing response data").runes.len() {
		1 => {
			if let Some(rune_stats) = &data.runes[0].rs_rune_stats {
				if let Some(rune_stats_runes) = &rune_stats.runes {
					if let Some(amount) = &rune_stats_runes.terms_amount {
						println!("{:?}", amount);
					}
				}
			}
		},
		_ => {println!("NO RUNE DATA");}
	}

	Ok(())
}