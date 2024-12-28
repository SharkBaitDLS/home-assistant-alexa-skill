use monostate::MustBe;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    #[allow(dead_code)]
    pub payload_version: MustBe!("3"),
    pub correlation_token: Option<String>,
}

#[derive(Derivative, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bearer {
    // Home assistant only supports bearer token skill auth
    #[serde(rename = "type")]
    pub token_type: MustBe!("BearerToken"),
    #[derivative(Debug = "ignore")]
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointConfig {
    pub scope: Bearer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    // Depending on the request type, the auth data will be in one of these
    // fields (or in the endpoint struct)
    pub grantee: Option<Bearer>,
    pub scope: Option<Bearer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Directive {
    pub header: Header,
    pub endpoint: Option<EndpointConfig>,
    pub payload: Option<Payload>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub directive: Directive,
}
