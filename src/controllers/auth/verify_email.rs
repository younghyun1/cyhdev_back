use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::State, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::{
    get_conn, get_transaction,
    models::user_tokens::UserToken,
    utils::{
        errors::errors::{ErrResp, ErrRespDat},
        gadgets::stopwatch::Stopwatch,
        server_init::server_state_def::ServerState,
    },
};

// request
#[derive(Deserialize)]
pub struct VerifyEmailForm {
    token_id: Uuid,
}

// response
#[derive(Serialize)]
pub struct VerifyEmailResponse {
    success: bool,
    data: VerifyEmailResponseData,
    meta: VerifyEmailResponseMeta,
}

#[derive(Serialize)]
pub struct VerifyEmailResponseData {
    message: String,
}

#[derive(Serialize)]
pub struct VerifyEmailResponseMeta {
    time_taken: String,
    timestamp: DateTime<Utc>,
}

// POST /api/auth/validate-email
pub async fn verify_email(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<VerifyEmailForm>,
) -> impl IntoResponse {
    let stopwatch: Stopwatch = Stopwatch::new("verify_email");
    let mut conn = get_conn!(&state, stopwatch);

    let token = match UserToken::get_by_id(&conn, body.token_id).await {
        Ok(Some(tok)) => tok,
        Ok(None) => {
            return ErrResp::from(
                ErrRespDat::USER_TOKEN_INVALID,
                &stopwatch,
                anyhow!("Invalid user token!"),
            )
            .into_response()
        }
        Err(e) => {
            error!("Could not get UserToken by ID: {:?}", e);
            return ErrResp::from(
                ErrRespDat::USER_TOKEN_INVALID,
                &stopwatch,
                anyhow!("Invalid user token!"),
            )
            .into_response();
        }
    };

    if token.is_used() {
        return ErrResp::from(
            ErrRespDat::USER_TOKEN_USED,
            &stopwatch,
            anyhow!("User token already used!"),
        )
        .into_response();
    }

    if token.is_expired() {
        return ErrResp::from(
            ErrRespDat::USER_TOKEN_EXPIRED,
            &stopwatch,
            anyhow!("User token expired at: {}", token.get_expired_time()),
        )
        .into_response();
    }

    let transaction = get_transaction!(conn, stopwatch);

    match token.validate_user_email(&transaction).await {
        Ok(_) => (),
        Err(e) => {
            error!("Could not execute SQL or user was already verified: {}", e);
            return ErrResp::from(
                ErrRespDat::USER_ALREADY_VERIFIED,
                &stopwatch,
                anyhow!("User already verified with this token!"),
            )
            .into_response();
        }
    };

    match transaction.commit().await {
        Ok(_) => (),
        Err(e) => {
            error!("Could not commit transaction: {}", e);
            return ErrResp::from(
                ErrRespDat::COULD_NOT_COMMIT_TRANSACTION,
                &stopwatch,
                anyhow!("Failed to commit transaction!"),
            )
            .into_response();
        }
    }

    let response = VerifyEmailResponse {
        success: true,
        data: VerifyEmailResponseData {
            message: "Email verification successful!".to_string(),
        },
        meta: VerifyEmailResponseMeta {
            time_taken: format!("{:?}", stopwatch.get_original_start().elapsed()),
            timestamp: Utc::now(),
        },
    };

    let serialized_response = match bincode::serialize(&response) {
        Ok(data) => data,
        Err(e) => {
            error!("Could not serialize response: {:?}", e);
            return ErrResp::from(
                ErrRespDat::COULD_NOT_SERIALIZE_BINCODE,
                &stopwatch,
                anyhow!("Failed to serialize response!"),
            )
            .into_response();
        }
    };

    match axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, "application/octet-stream")
        .body(axum::body::Body::from(serialized_response))
    {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to build response: {:?}", e);
            return ErrResp::from(ErrRespDat::COULD_NOT_BUILD_RESPONSE, &stopwatch, anyhow!(e))
                .into_response();
        }
    }
}
