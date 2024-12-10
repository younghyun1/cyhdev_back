use std::sync::Arc;

use axum::{middleware::from_fn, routing::post};
use tower_http::compression::CompressionLayer;

use crate::utils::server_init::server_state_def::ServerState;

use super::{auth::signup::signup, middleware::request_response_info::print_request_info};

pub fn generate_router(state: &Arc<ServerState>) -> axum::Router {
    axum::Router::new()
        .route("/auth/signup", post(signup))
        .layer(CompressionLayer::new())
        .layer(from_fn(print_request_info))
        .with_state(Arc::clone(state))
}
