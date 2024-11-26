use axum::response::IntoResponse;
use bitcode::Encode;
use chrono::Utc;

use crate::utils::gadgets::stopwatch::Stopwatch;

#[derive(Encode, Clone, Debug)]
pub struct SvrErrorResponse {
    success: bool,
    data: SvrErrorRespData,
    meta: SvrErrorRespMeta,
}

#[derive(Encode, Clone, Debug)]
pub struct SvrErrorRespData {
    code: u8,
    message: &'static str,
    status_code: u16,
}

#[derive(Encode, Clone, Debug)]
pub struct SvrErrorRespMeta {
    time_taken: String,
    timestamp: String,
}

impl IntoResponse for SvrErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let serialized_body = bitcode::encode(&self);
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
}

impl SvrErrorResponse {
    pub fn from(dat: SvrErrorRespData, start: Stopwatch) -> Self {
        SvrErrorResponse {
            success: false,
            data: dat,
            meta: SvrErrorRespMeta {
                time_taken: format!("{:?}", start.get_original_start().elapsed()),
                timestamp: Utc::now().to_rfc3339(),
            },
        }
    }
}

impl SvrErrorRespData {
    pub const COULD_NOT_GET_CONN_FROM_POOL: SvrErrorRespData = SvrErrorRespData {
        code: 1,
        message: "Could not get connection from pool!",
        status_code: 404,
    };
}
