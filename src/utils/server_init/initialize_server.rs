use chrono::{DateTime, Utc};

use crate::utils::gadgets::stopwatch::Stopwatch;

use super::{
    server_init_funcs::{
        https_redirector::redirect_http_to_https, initialize_crypto::init_crypto,
        load_cert_config::load_certs,
    },
    server_state_def::ServerState,
};

pub async fn init_server(
    mut stopwatch: &mut Stopwatch,
    server_start_time: DateTime<Utc>,
) -> anyhow::Result<()> {
    // initialize crypto
    init_crypto()?;
    // load certs
    let cert_config = load_certs().await?;
    stopwatch.click("crypto initialized, certs loaded");

    // set up http -> https redirect server
    tokio::spawn(redirect_http_to_https());
    stopwatch.click("HTTPS redirection server online.");

    // initialize server state
    let state = ServerState::new(&mut stopwatch, server_start_time)?;
    stopwatch.click("server state initialized");

    Ok(())
}
