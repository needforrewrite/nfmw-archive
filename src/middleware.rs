use axum::{body::Body, http::Request, middleware::Next, response::Response};
use log::debug;

pub async fn debug_logger(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let query = uri
        .query()
        .map(|q| format!(" ?{q}"))
        .unwrap_or_default();

    let (parts, body) = req.into_parts();
    let req_bytes = axum::body::to_bytes(body, 1024 * 1024)
        .await
        .unwrap_or_default();

    let body_log = if req_bytes.is_empty() {
        String::new()
    } else {
        match std::str::from_utf8(&req_bytes) {
            Ok(s) => format!("\n  body: {s}"),
            Err(_) => format!("\n  body: <{} bytes binary>", req_bytes.len()),
        }
    };

    debug!("{} {}{}{}", method, uri.path(), query, body_log);

    let req = Request::from_parts(parts, Body::from(req_bytes));
    let response = next.run(req).await;

    let status = response.status();
    let (resp_parts, resp_body) = response.into_parts();
    let resp_bytes = axum::body::to_bytes(resp_body, 1024 * 1024)
        .await
        .unwrap_or_default();

    let resp_body_log = if resp_bytes.is_empty() {
        String::new()
    } else {
        match std::str::from_utf8(&resp_bytes) {
            Ok(s) => format!(" {s}"),
            Err(_) => format!(" <{} bytes binary>", resp_bytes.len()),
        }
    };

    debug!("  -> {}{}", status, resp_body_log);

    Response::from_parts(resp_parts, Body::from(resp_bytes))
}
