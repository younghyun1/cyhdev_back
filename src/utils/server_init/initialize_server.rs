use std::sync::Arc;

use anyhow::anyhow;
use chrono::{DateTime, Utc};

use crate::{controllers::router::generate_router, utils::gadgets::stopwatch::Stopwatch};

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
    let state = Arc::new(ServerState::new(&mut stopwatch, server_start_time)?);
    stopwatch.click("server state initialized");

    // test connection pool
    let conn = state.get_conn().await?;
    stopwatch.click("connection allocated from pool");

    // validate DB connection and protocol
    let ver_string = conn
        .query_one("SELECT VERSION();", &[])
        .await
        .map_err(|e| anyhow!("failed to execute test query on the database: {:?}", e))?
        .get::<usize, String>(0);
    drop(conn);
    stopwatch.click(&format!("DB connection verified: {}; latency", ver_string));
    drop(ver_string);

    // define router
    let router = generate_router(&state).await;
    stopwatch.click("routers defined");

    stopwatch.total("server started in");

    // serve server
    match axum_server::bind_rustls(state.get_socket_addr(), cert_config)
        .serve(router.into_make_service())
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Axum could not serve app: {:?}", e));
        }
    };

    Ok(())
}
