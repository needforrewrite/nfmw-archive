use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use hmac::{Hmac, Mac};
use rand_core::TryRngCore;
use rand_core::OsRng;
use sha2::Digest;

type HmacSha256 = Hmac<sha2::Sha256>;

pub fn generate_base64_authentication_token() -> String {
    let mut token_bytes = [0u8; 64];
    OsRng.try_fill_bytes(&mut token_bytes).unwrap();
    BASE64_STANDARD.encode(&token_bytes)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(token.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Verifies an HMAC-SHA256 signature in constant time. `signature` is hex; a
/// malformed one is simply a failed verification, not an error worth
/// distinguishing to the caller.
pub fn verify_hmac_sha256(secret: &[u8], message: &[u8], signature: &str) -> bool {
    let Ok(signature) = hex::decode(signature) else {
        return false;
    };

    let mut mac = HmacSha256::new_from_slice(secret).expect("hmac accepts keys of any length");
    mac.update(message);
    mac.verify_slice(&signature).is_ok()
}
