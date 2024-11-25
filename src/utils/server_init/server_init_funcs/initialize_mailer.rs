use anyhow::{anyhow, Result};
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};

use super::load_env_vars::get_env_var;

pub fn init_mailer() -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let creds = Credentials::new(
        get_env_var("SMTP_USERNAME")?.to_owned(),
        get_env_var("SMTP_PASSWORD")?.to_owned(),
    );

    let mailer: AsyncSmtpTransport<Tokio1Executor> = match AsyncSmtpTransport::<Tokio1Executor>::relay("email-smtp.ap-northeast-2.amazonaws.com") {
        Ok(relay) => relay,
        Err(e) => {
            return Err(anyhow!("Could not build SMTP relay: {:?}", e));
        }
    }
    .credentials(creds)
    .build();

    Ok(mailer)
}
