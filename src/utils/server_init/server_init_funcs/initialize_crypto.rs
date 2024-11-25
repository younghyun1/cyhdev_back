use anyhow::anyhow;
use rustls::crypto::CryptoProvider;

pub fn init_crypto() -> anyhow::Result<()> {
    match CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider()) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(anyhow!("Could not install aws_lc_rs: {:?}", e));
        }
    }
}
