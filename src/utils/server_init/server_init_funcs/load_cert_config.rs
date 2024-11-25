use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use axum_server::tls_rustls::RustlsConfig;

use super::load_env_vars::get_env_var;

pub async fn load_certs() -> Result<RustlsConfig> {
    match RustlsConfig::from_pem_file(
        PathBuf::from_str(get_env_var("CERT_DIR")?.as_ref())?,
        PathBuf::from_str(get_env_var("KEY_DIR")?.as_ref())?,
    )
    .await
    {
        Ok(cfg) => Ok(cfg),
        Err(e) => Err(anyhow!("Failed to load .pem keys: {:?}", e)),
    }
}
