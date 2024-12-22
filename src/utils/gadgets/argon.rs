use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap_or_else(|e| panic!("Failed to hash password: {:?}", e))
        .to_string();

    password_hash
}

pub fn verify_password(hash: String, password: String) -> Result<bool> {
    let parsed_hash = PasswordHash::new(&hash).map_err(|e| anyhow::anyhow!(e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
