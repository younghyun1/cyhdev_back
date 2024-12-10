use std::net::SocketAddr;

use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    BoxError,
};
use dotenvy::var;

#[derive(Clone, Copy)]
pub struct Ports {
    http: u16,
    https: u16,
}

impl Ports {
    fn new(http_port: u16, https_port: u16) -> Self {
        Ports {
            http: http_port,
            https: https_port,
        }
    }
}

pub async fn redirect_http_to_https() {
    let https_port = var("HOST_PORT")
        .ok()
        .and_then(|port_str| port_str.parse::<u16>().ok())
        .unwrap_or(443u16);

    let ports = Ports::new(80u16, https_port);

    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some(unsafe { "/".parse().unwrap_unchecked() });
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            match listener.local_addr() {
                Ok(local_addr) => tracing::debug!("listening on {}", local_addr),
                Err(e) => tracing::error!("failed to get local address: {}", e),
            }

            if let Err(e) = axum::serve(listener, redirect.into_make_service()).await {
                tracing::error!("axum server error: {}", e);
            }
        }
        Err(e) => tracing::error!("failed to bind to address: {}", e),
    }
}
