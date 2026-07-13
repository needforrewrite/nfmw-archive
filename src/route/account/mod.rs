pub mod oauth2;
pub mod local;
pub mod service_key;

pub fn validate_username(username: &str) -> Result<(), String> {
    if username.len() < 3 || username.len() > 32 {
        return Err("Username must be between 3 and 32 characters long".to_string());
    }

    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err("Username can only contain alphanumeric characters, underscores, and hyphens".to_string());
    }

    if username.chars().all(|c| c == '_' || c == '-') {
        return Err("Username cannot consist only of underscores and hyphens".to_string());
    }

    Ok(())
}