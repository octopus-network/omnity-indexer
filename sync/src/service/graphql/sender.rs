use reqwest::Client;
use graphql_client::{GraphQLQuery, Response};

#[allow(non_camel_case_types)]
type json = serde_json::Value;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.json",
    query_path = "src/sender.graphql",
    response_derives = "Debug",
)]
pub struct MyQuery;

#[tokio::main]
async fn query_sender() -> Result<(), Box<dyn std::error::Error>> {
	let variables: my_query::Variables = my_query::Variables{address: "a10c9b71bf15458093eeb883cc4cd15078bb34a55ae78ea2480fe2927c1102ca".to_string()};
	let request_body = MyQuery::build_query(variables);
	let client = Client::new();
	let response = client
		.post("https://hasura-secondary-graphql-engine-2252klcbva-uc.a.run.app/v1/graphql")
		.header("x-hasura-admin-secret", "C26sQVLsq3Adzd7CoHfv")
		.json(&request_body)
		.send()
		.await.expect("Error sending request");

	let response_body: Response<my_query::ResponseData> = response.json().await.expect("Error deserializing response");
	let data = response_body.data.expect("Missing response data");
	println!("{:?}",data.transactions[0].transaction["inputs"][0]["address"]);

	Ok(())
}