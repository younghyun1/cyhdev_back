use std::sync::Arc;

use axum::{debug_handler, extract::State, response::IntoResponse, Json};

use crate::{models::users::UserForm, utils::server_init::server_state_def::ServerState};

#[debug_handler]
pub async fn signup(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<UserForm>,
) -> impl IntoResponse {
}
