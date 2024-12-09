// POST /auth/verify-email

use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::State, response::IntoResponse, Json};
use tracing::error;
use uuid::Uuid;

use crate::{
    get_conn,
    models::user_tokens::UserToken,
    utils::{
        errors::errors::{ErrResp, ErrRespDat},
        gadgets::stopwatch::Stopwatch,
        server_init::server_state_def::ServerState,
    },
};

pub struct VerifyEmailForm {
    token_id: Uuid,
}

pub async fn verify_email(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<VerifyEmailForm>,
) -> impl IntoResponse {
    let stopwatch: Stopwatch = Stopwatch::new("verify_email");
    let conn = get_conn!(&state, stopwatch);

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
    
    

    todo!()
}
