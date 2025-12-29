use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use rand_core::TryRngCore;
use rand_core::OsRng;
use sha2::{Sha512, Digest};
use subtle::ConstantTimeEq;

pub fn generate_salt() -> [u8; 64] {
    let mut salt = [0u8; 64];
    OsRng.try_fill_bytes(&mut salt).unwrap();
    salt
}

pub fn hash_password(password: &str, salt: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(salt);
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn generate_base64_authentication_token() -> String {
    let mut token_bytes = [0u8; 64];
    OsRng.try_fill_bytes(&mut token_bytes).unwrap();
    BASE64_STANDARD.encode(&token_bytes)
}

/// When a client provides an authentication token, it first salts it with a randomly generated salt on the client's end per request.
/// Once that salt has been used, it cannot be reused. This prevents any middleman from reusing a captured token.
/// The server then verifies the salted token by hashing it with the same salt and comparing it to the expected hash.
pub fn validate_salted_authentication_token(token: &str, salt: &[u8], expected_hash: &str) -> bool {
    let mut hasher = Sha512::new();
    hasher.update(salt);
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    let computed_hash = hex::encode(result);
    computed_hash.as_bytes().ct_eq(expected_hash.as_bytes()).unwrap_u8() == 1
}