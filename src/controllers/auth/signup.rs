use std::sync::Arc;

use anyhow::anyhow;
use axum::{debug_handler, extract::State, response::IntoResponse, Json};
use uuid::Uuid;

use crate::{
    get_conn, get_transaction,
    models::{
        user_tokens::{UserTokenForm, SIGNUP_EMAIL_VALIDATE},
        users::{User, UserForm},
    },
    utils::{
        errors::errors::{ErrResp, ErrRespDat},
        gadgets::stopwatch::{self, Stopwatch},
        server_init::server_state_def::ServerState,
    },
};

pub async fn signup(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<UserForm>,
) -> impl IntoResponse {
    let stopwatch: Stopwatch = Stopwatch::new("");

    if !state.email_regex().is_match(&body.user_email) {
        return ErrResp::from(ErrRespDat::WRONG_EMAIL_FORMAT, &stopwatch, anyhow!(""))
            .into_response();
    }

    if !state.pw_regex().is_match(&body.user_password) {
        return ErrResp::from(ErrRespDat::WRONG_PW_FORMAT, &stopwatch, anyhow!("")).into_response();
    }

    let mut conn = get_conn!(&state, &stopwatch);
    let transaction = get_transaction!(conn, &stopwatch);

    // insert new user into DB
    let returned_user: User = match body.insert(&transaction).await {
        Ok(user) => user,
        Err(e) => {
            return ErrResp::from(ErrRespDat::COULD_NOT_INSERT_USER, &stopwatch, anyhow!(e))
                .into_response()
        }
    };

    let user_token_id: uuid::Uuid = Uuid::new_v4();

    UserTokenForm {
        user_token_user_id: user_token_id,
        user_token_type: SIGNUP_EMAIL_VALIDATE.to_owned(),
        user_token_value: todo!(),
        user_token_expires_at: todo!(),
    };

    todo!()
}
