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

#[tokio::main]
pub async fn query_sender(address: &str) -> Result<String, anyhow::Error> {
	let variables: sender_query::Variables = sender_query::Variables {
		address: address.to_string(),
	};
	let request_body = SenderQuery::build_query(variables);
	let client = Client::new();
	let response = client
		.post("https://hasura-secondary-graphql-engine-2252klcbva-uc.a.run.app/v1/graphql")
		.header("x-hasura-admin-secret", "C26sQVLsq3Adzd7CoHfv")
		.json(&request_body)
		.send()
		.await
		.expect("Error sending request");

	let response_body: Response<sender_query::ResponseData> =
		response.json().await.expect("Error deserializing response");
	let data = response_body.data.expect("Missing response data");

	Ok(data.transactions[0].transaction["inputs"][0]["address"].to_string())
}
