use reqwest::{Response, StatusCode};
use serde::Serialize;
use serde_json::value::RawValue;
use tracing::{event, instrument, Level};
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorHeader {
    namespace: String,
    name: String,
    message_id: String,
    correlation_token: String,
    payload_version: String,
}

impl ErrorHeader {
    fn new(correlation_token: String) -> Self {
        ErrorHeader {
            namespace: "Alexa".to_owned(),
            name: "ErrorResponse".to_owned(),
            message_id: Uuid::new_v4().to_string(),
            correlation_token,
            payload_version: "3".to_owned(),
        }
    }
}

#[derive(Serialize)]
enum ErrorType {
    #[serde(rename = "INVALID_AUTHORIZATION_CREDENTIAL")]
    InvalidToken,
    #[serde(rename = "INTERNAL_ERROR")]
    System,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorPayload {
    #[serde(rename = "type")]
    error_type: ErrorType,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorEvent {
    header: ErrorHeader,
    payload: ErrorPayload,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    event: ErrorEvent,
}

#[instrument]
pub async fn create(correlation_token: String, response: Response) -> Option<Box<RawValue>> {
    match response.status() {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            let message = response
                .bytes()
                .await
                .ok()
                .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
                .unwrap_or("Invalid access token".to_owned());

            event!(Level::WARN, "Unauthorized error: {message}");

            serde_json::value::to_raw_value(&ErrorResponse {
                event: ErrorEvent {
                    header: ErrorHeader::new(correlation_token),
                    payload: ErrorPayload {
                        error_type: ErrorType::InvalidToken,
                        message,
                    },
                },
            })
            .ok()
        }
        code if code.as_u16() >= 500 => {
            let message = response
                .bytes()
                .await
                .ok()
                .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
                .unwrap_or("Unexpected error".to_owned());

            event!(Level::ERROR, "System error: {message}");

            serde_json::value::to_raw_value(&ErrorResponse {
                event: ErrorEvent {
                    header: ErrorHeader::new(correlation_token),
                    payload: ErrorPayload {
                        error_type: ErrorType::System,
                        message,
                    },
                },
            })
            .ok()
        }
        // Pass through successful responses from the Home Assistant backend raw
        _ => response.json().await.ok(),
    }
}
