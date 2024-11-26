use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde_derive::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct ErrorResponse {
    success: bool,
    data: ErrorResponseData,
    meta: ErrorResponseMeta,
}

#[derive(Serialize, Clone, Debug)]
pub struct ErrorResponseData {
    code: u8,
    message: &'static str,
    status_code: u16,
}

#[derive(Serialize, Clone, Debug)]
pub struct ErrorResponseMeta {
    time_taken: String,
    timestamp: DateTime<Utc>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        match bincode::serialize(&self) {
            Ok(serialized_body) => {
                let response = match axum::response::Response::builder()
                    .status(self.data.status_code)
                    .header("Content-Type", "application/octet-stream")
                    .body(axum::body::Body::from(serialized_body))
                {
                    Ok(resp) => resp,
                    Err(e) => panic!("Failed to serialize the response: {:?}", e),
                };
                response
            }
            Err(e) => panic!("Failed to serialize the response: {:?}", e),
        }
    }
}
