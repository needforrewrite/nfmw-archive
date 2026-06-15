use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rand_core::TryRngCore;
use rand_core::OsRng;
use sha2::Digest;

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