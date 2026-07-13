use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{FromRef, FromRequest, Request},
};
use serde::de::DeserializeOwned;

use crate::{
    config::Config,
    crypto::{sha256_hex, verify_hmac_sha256},
    route::error::AppError,
};

const SIGNATURE_SCHEME: &str = "NFMW-HMAC-SHA256";

/// An authenticated peer service (a lobby, a game server), identified by the
/// config entry whose secret signed the request.
pub struct ServiceCaller {
    pub service_id: String,
}

/// A JSON body from a service that proved its identity by signing the request.
///
/// The signature covers the method, path, query, target service, timestamp and
/// body, so a captured signature cannot be replayed against a different route or
/// a different service, and the body cannot be swapped under it.
///
/// Note this is authentication, not replay protection: a signature can be
/// replayed verbatim inside the clock-skew window. Endpoints reached this way
/// must therefore be idempotent or single-use — `/auth/service-key/validate` is
/// the latter, since redeeming a key destroys it.
pub struct SignedJson<T> {
    pub caller: ServiceCaller,
    pub body: T,
}

struct SignatureHeader {
    service: String,
    timestamp: i64,
    signature: String,
}

/// Parses `NFMW-HMAC-SHA256 service=lobby,ts=1720900000,sig=<hex>`.
fn parse_signature_header(header: &str) -> Option<SignatureHeader> {
    let params = header.strip_prefix(SIGNATURE_SCHEME)?.trim();

    let mut service = None;
    let mut timestamp = None;
    let mut signature = None;

    for param in params.split(',') {
        let (key, value) = param.trim().split_once('=')?;
        match key {
            "service" => service = Some(value.to_owned()),
            "ts" => timestamp = Some(value.parse().ok()?),
            "sig" => signature = Some(value.to_owned()),
            _ => return None,
        }
    }

    Some(SignatureHeader {
        service: service?,
        timestamp: timestamp?,
        signature: signature?,
    })
}

impl<T, S> FromRequest<S> for SignedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
    Arc<Config>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let config = Arc::<Config>::from_ref(state);

        let (parts, body) = req.into_parts();

        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_signature_header)
            .ok_or(AppError::Unauthorized)?;

        // Unknown service and bad signature are the same answer on purpose: this
        // endpoint should not confirm which services exist.
        let service = config.service(&header.service).ok_or(AppError::Unauthorized)?;

        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        if (now - header.timestamp).abs() > config.hmac_max_skew_seconds {
            return Err(AppError::Unauthorized);
        }

        let path_and_query = parts
            .uri
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or_else(|| parts.uri.path());

        let bytes = Bytes::from_request(Request::from_parts(parts.clone(), body), state)
            .await
            .map_err(|_| AppError::BadRequest("could not read request body".to_string()))?;

        let string_to_sign = format!(
            "{}\n{}\n{}\n{}\n{}",
            parts.method,
            path_and_query,
            service.id,
            header.timestamp,
            sha256_hex(&bytes),
        );

        let authenticated = service.hmac_secrets.iter().any(|secret| {
            let Ok(secret) = hex::decode(secret) else {
                return false;
            };
            verify_hmac_sha256(&secret, string_to_sign.as_bytes(), &header.signature)
        });

        if !authenticated {
            return Err(AppError::Unauthorized);
        }

        let body = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::BadRequest(format!("invalid request body: {}", e)))?;

        Ok(SignedJson {
            caller: ServiceCaller { service_id: service.id.clone() },
            body,
        })
    }
}
