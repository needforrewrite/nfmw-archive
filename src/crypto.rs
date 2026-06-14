use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rand_core::TryRngCore;
use rand_core::OsRng;

pub fn generate_base64_authentication_token() -> String {
    let mut token_bytes = [0u8; 64];
    OsRng.try_fill_bytes(&mut token_bytes).unwrap();
    BASE64_STANDARD.encode(&token_bytes)
}