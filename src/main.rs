mod request;
mod response;

#[macro_use]
extern crate derivative;

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use request::Request;
use reqwest::Client;
use serde_json::value::RawValue;
use tracing::instrument;
use tracing_subscriber::EnvFilter;

#[instrument(level = "debug")]
async fn function_handler(
    event: LambdaEvent<Request>,
    client: &Client,
    base_url: &str,
) -> Result<Box<RawValue>, Error> {
    let directive = &event.payload.directive;

    // The auth token will be in one of these 3 places depending on the type of request
    let token = directive
        .endpoint
        .as_ref()
        .map(|endpoint| &endpoint.scope)
        .or(directive
            .payload
            .as_ref()
            .and_then(|payload| payload.scope.as_ref().or(payload.grantee.as_ref())));

    match token {
        None => Err("No authorization token present".into()),
        Some(bearer) => {
            // Send the request to the self-hosted HA instance
            let response = client
                .post(format!("{base_url}/api/alexa/smart_home"))
                .bearer_auth(&bearer.token)
                .json(&event.payload)
                .send()
                .await?;

            // Successful responses get passed through raw, errors get marshalled
            response::create(directive.header.correlation_token.to_owned(), response)
                .await
                .ok_or("Could not serialize a response".into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        // defer to the RUST_LOG environment variable for filtering/log level
        .with_env_filter(EnvFilter::from_default_env())
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // The full HTTP(S) URL for your Home Assistant instance
    let base_url = std::env::var("BASE_URL").expect("BASE_URL must be in the environment");

    // Retain a Reqwest client in the warmed Lambda runtime independently of invocations
    let client = Client::builder().build()?;

    run(service_fn(|event| {
        function_handler(event, &client, &base_url)
    }))
    .await
}
