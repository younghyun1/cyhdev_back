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
    // time measurement
    let stopwatch: Stopwatch = Stopwatch::new("");

    // check if email is valid form
    if !state.email_regex().is_match(&body.user_email) {
        return ErrResp::from(ErrRespDat::WRONG_EMAIL_FORMAT, &stopwatch, anyhow!(""))
            .into_response();
    }

    // check if password is valid form (At least 8 characters and includes uppercase, lowercase, number, and special characters among: [@, $, !, %, *, ?, &, #])
    if !state.pw_regex().is_match(&body.user_password) {
        return ErrResp::from(ErrRespDat::WRONG_PW_FORMAT, &stopwatch, anyhow!("")).into_response();
    }

    // get database connection and transaction objects
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

    // new token's PKEY (email_validation)
    let user_token_id: uuid::Uuid = Uuid::new_v4();

    // new token insertion form (email_validation)
    let user_token_form = UserTokenForm {
        user_token_user_id: returned_user.get_id(),
        user_token_type: SIGNUP_EMAIL_VALIDATE.to_owned(),
        user_token_value: user_token_id,
        user_token_expires_at: returned_user.get_created_at() + chrono::Duration::days(1),
    };

    // insert into DB and get token (email_validation)
    let returned_token = match user_token_form.insert(&transaction).await {
        Ok(token) => token,
        Err(e) => {
            return ErrResp::from(
                ErrRespDat::COULD_NOT_INSERT_USER_TOKEN,
                &stopwatch,
                anyhow!(e),
            )
            .into_response()
        }
    };
    
    
}
