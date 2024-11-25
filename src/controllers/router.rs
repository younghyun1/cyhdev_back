use std::sync::Arc;

use axum::middleware::from_fn;
use tower_http::compression::CompressionLayer;

use crate::utils::server_init::server_state_def::ServerState;

use super::middleware::request_response_info::print_request_info;

pub fn generate_router(state: &Arc<ServerState>) -> axum::Router {
    axum::Router::new()
        .layer(CompressionLayer::new())
        .layer(from_fn(print_request_info))
        .with_state(Arc::clone(state))
}
