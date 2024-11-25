use std::path::PathBuf;

use anyhow::anyhow;
use dotenvy::{dotenv, var};

/// load environment variables from .env using 'dotenvy' crate
pub fn load_env() -> anyhow::Result<PathBuf> {
    match dotenv() {
        Ok(path_buf) => Ok(path_buf),
        Err(e) => {
            return Err(anyhow!(
                "Dotenvy could not load .env file: {}",
                e.to_string()
            ));
        }
    }
}

pub fn get_env_var(key: &str) -> anyhow::Result<String> {
    match var(key) {
        Ok(value) => Ok(value),
        Err(e) => Err(anyhow!("Could not find var {}: {:?}", key, e)),
    }
}
