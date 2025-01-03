use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::State, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use lettre::{message::Mailbox, AsyncTransport, Message};
use serde_derive::Serialize;
use tokio_postgres::error::SqlState;
use tracing::error;
use uuid::Uuid;

use crate::{
    get_conn, get_transaction,
    models::{
        consts::SMTP_EMAIL,
        user_tokens::{UserToken, UserTokenForm, SIGNUP_EMAIL_VALIDATE},
        users::{UserForm, UserTruncated},
    },
    utils::{
        errors::errors::{ErrResp, ErrRespDat},
        gadgets::{regex::pw_regex_custom, stopwatch::Stopwatch},
        serde::serialize_to_response::serialize_to_response,
        server_init::server_state_def::ServerState,
    },
};

// POST /auth/signup
// Request body JSON for user signup
// pub struct UserForm {
//     pub user_screen_name: String,
//     pub user_email: String,
//     pub user_password: String,
// }

#[derive(Serialize)]
pub struct SignupResponse {
    success: bool,
    data: SignupResponseData,
    meta: SignupResponseMeta,
}

#[derive(Serialize)]
pub struct SignupResponseData {
    user: UserTruncated,
}

#[derive(Serialize)]
pub struct SignupResponseMeta {
    time_taken: String,
    timestamp: DateTime<Utc>,
}

// POST /api/auth/signup
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
    if !pw_regex_custom(&body.user_password) {
        return ErrResp::from(ErrRespDat::WRONG_PW_FORMAT, &stopwatch, anyhow!("")).into_response();
    }

    // get database connection and transaction objects
    let mut conn = get_conn!(&state, &stopwatch);
    let transaction = get_transaction!(conn, &stopwatch);

    // insert new user into DB
    let returned_user: UserTruncated = match body.insert(&transaction).await {
        Ok(user) => user,
        Err(e) => match e.as_db_error() {
            Some(db_error) => match *db_error.code() {
                SqlState::UNIQUE_VIOLATION => {
                    return ErrResp::from(
                        ErrRespDat::USER_ALREADY_EXISTS,
                        &stopwatch,
                        anyhow!("User already exists! Please use another email and screen name."),
                    )
                    .into_response();
                }
                _ => {
                    return ErrResp::from(ErrRespDat::COULD_NOT_INSERT_USER, &stopwatch, anyhow!(e))
                        .into_response()
                }
            },
            None => {
                return ErrResp::from(ErrRespDat::COULD_NOT_INSERT_USER, &stopwatch, anyhow!(e))
                    .into_response();
            }
        },
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
    let returned_token: UserToken = match user_token_form.insert(&transaction).await {
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

    let returned_token_id = returned_token.get_id();
    drop(returned_token);

    // commit transaction
    match transaction.commit().await {
        Ok(_) => {
            tokio::spawn({
                // if successfully committed onto DB, then send email in separate thread
                let state = Arc::clone(&state);
                let body_user_email = body.user_email.clone();

                // construct email
                async move {
                    let email: Message = match Message::builder()
                        .from(unsafe {SMTP_EMAIL.parse().unwrap_unchecked()})
                        .to(match body_user_email.parse::<Mailbox>() {
                            Ok(mb) => mb,
                            Err(e) => {
                                error!("Could not parse body email: {:?}", e);
                                return;
                            },
                        })
                        .subject("Email Verification for cyhdev.com forums!")
                        .body(format!(
                            "Please verify your email by clicking on the following link: https://www.cyhdev.com/auth/verify_email?email_token={}",
                            returned_token_id
                        )) {
                            Ok(email) => email,
                            Err(e) => {
                                error!("Could not construct email: {:?}", e);
                                return;
                            },
                        };

                    // send email on shared email client
                    match state.get_mailer().send(email).await {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Could not send mail: {:?}", e);
                        }
                    };
                }
            });

            // serialize user w. truncated password hash for return
            let signup_response = SignupResponse {
                success: true,
                data: SignupResponseData {
                    user: returned_user,
                },
                meta: SignupResponseMeta {
                    time_taken: format!("{:?}", stopwatch.get_original_start().elapsed()),
                    timestamp: Utc::now(),
                },
            };

            serialize_to_response(&signup_response, &stopwatch)
        }

        Err(e) => {
            error!("Could not commit transaction: {:?}", e);
            ErrResp::from(
                ErrRespDat::COULD_NOT_COMMIT_TRANSACTION,
                &stopwatch,
                anyhow!(e),
            )
            .into_response()
        }
    }
}
