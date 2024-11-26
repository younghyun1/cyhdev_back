use std::sync::Arc;

use axum::{debug_handler, extract::State, response::IntoResponse, Json};

use crate::{
    models::users::UserForm,
    utils::{
        errors::errors::{SvrErrorRespData, SvrErrorResponse},
        gadgets::stopwatch::{self, Stopwatch},
        server_init::server_state_def::ServerState,
    },
};

#[debug_handler]
pub async fn signup(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<UserForm>,
) -> impl IntoResponse {
    let stopwatch: Stopwatch = Stopwatch::new("");

    SvrErrorResponse::from(SvrErrorRespData::COULD_NOT_GET_CONN_FROM_POOL, stopwatch)
        .into_response()
}
