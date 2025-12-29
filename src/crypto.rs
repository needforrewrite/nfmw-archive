use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use rand_core::TryRngCore;
use rand_core::OsRng;

pub fn generate_salt() -> [u8; 64] {
    let mut salt = [0u8; 64];
    OsRng.try_fill_bytes(&mut salt).unwrap();
    salt
}

pub fn hash_password(password: &str, salt: &[u8]) -> String {
    use sha2::{Sha512, Digest};
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