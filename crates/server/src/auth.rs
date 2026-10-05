use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::SaltString;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::from_b64(&Uuid::new_v4().simple().to_string())
        .map_err(|_| "could not create password salt".to_string())?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "could not hash password".to_string())
}

pub fn new_session_token() -> String {
    Uuid::new_v4().simple().to_string() + &Uuid::new_v4().simple().to_string()
}

pub fn new_qr_login_token() -> String {
    Uuid::new_v4().simple().to_string() + &Uuid::new_v4().simple().to_string()
}

pub const QR_LOGIN_MS: i64 = 2 * 60 * 1000;

pub fn new_access_key() -> String {
    let raw = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    raw.as_bytes()
        .chunks(8)
        .map(|chunk| std::str::from_utf8(chunk).expect("hex key is ascii"))
        .collect::<Vec<_>>()
        .join("-")
}

pub fn access_key_hash(key: &str) -> Option<String> {
    let mut normalized = String::with_capacity(64);
    for ch in key.chars() {
        if ch.is_ascii_hexdigit() {
            normalized.push(ch.to_ascii_lowercase());
        } else if ch != '-' && !ch.is_ascii_whitespace() {
            return None;
        }
    }
    if normalized.len() != 64 {
        return None;
    }
    Some(format!("{:x}", Sha256::digest(normalized.as_bytes())))
}

pub const SESSION_MS: i64 = 30 * 24 * 60 * 60 * 1000;
pub const COOKIE_NAME: &str = "larptrix_session";

#[cfg(test)]
mod tests {
    use super::{access_key_hash, new_access_key};

    #[test]
    fn generated_access_key_is_random_and_format_insensitive() {
        let key = new_access_key();
        assert_eq!(key.len(), 71);
        assert_eq!(key.matches('-').count(), 7);
        assert_eq!(
            access_key_hash(&key),
            access_key_hash(&key.replace('-', " "))
        );
        assert!(access_key_hash("short-key").is_none());
    }
}
