use axum::{extract::Request, middleware::Next, response::Response};
use tracing::info;

pub async fn print_request_info(request: Request, next: Next) -> Response {
    let start = tokio::time::Instant::now();
    let (method, uri, version) = (
        request.method().clone(),
        request.uri().clone(),
        request.version(),
    );

    let ip_str = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|header_value| header_value.to_str().ok())
        .map(|ip| ip.to_owned())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|header_value| header_value.to_str().ok())
                .map(|ip| ip.to_owned())
        })
        .unwrap_or_else(|| "unknown".to_owned());

    info!("{} {} {:?} from {}", method, uri, version, ip_str);

    let response = next.run(request).await;

    info!(
        "{} {} {:?}: {} in {:?}",
        method,
        uri,
        version,
        response.status(),
        start.elapsed()
    );

    response
}
