use crate::utils::{
    errors::errors::{ErrResp, ErrRespDat},
    gadgets::stopwatch::Stopwatch,
};

use anyhow::anyhow;
use axum::response::IntoResponse;
use tracing::error;

pub fn serialize_to_response<T: serde::Serialize>(
    value: &T,
    stopwatch: &Stopwatch,
) -> axum::http::Response<axum::body::Body> {
    let serialized_data = match bincode::serialize(value) {
        Ok(data) => data,
        Err(e) => {
            error!("Could not serialize value: {:?}", e);
            return ErrResp::from(
                ErrRespDat::COULD_NOT_SERIALIZE_BINCODE,
                stopwatch,
                anyhow!("Failed to serialize value!"),
            )
            .into_response();
        }
    };

    match axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, "application/octet-stream")
        .body(axum::body::Body::from(serialized_data))
    {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to build response: {:?}", e);
            ErrResp::from(ErrRespDat::COULD_NOT_BUILD_RESPONSE, stopwatch, anyhow!(e))
                .into_response()
        }
    }
}
