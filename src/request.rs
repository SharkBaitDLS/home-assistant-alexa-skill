use monostate::MustBe;
use serde::{Deserialize, Serialize};
use serde_json::value::{RawValue, Value};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub message_id: String,
    pub namespace: String,
    pub name: String,
    pub payload_version: MustBe!("3"),
    pub correlation_token: String,
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bearer {
    // Home assistant only supports bearer token skill auth
    #[serde(rename = "type")]
    pub token_type: MustBe!("BearerToken"),
    #[derivative(Debug = "ignore")]
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointConfig {
    pub scope: Bearer,
    pub endpoint_id: String,
    pub cookie: Box<RawValue>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    // Depending on the request type, the auth data will be in one of these
    // fields (or in the endpoint struct)
    pub grantee: Option<Bearer>,
    pub scope: Option<Bearer>,

    // There's sometimes contextual data for the actual smart home request
    // in here that needs to be forwarded to the HA instance, but we don't care
    // about it otherwise so it doesn't need to be modeled
    #[serde(flatten)]
    rest: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Directive {
    pub header: Header,
    pub endpoint: Option<EndpointConfig>,
    pub payload: Option<Payload>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub directive: Directive,
}
