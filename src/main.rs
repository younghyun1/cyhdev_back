pub mod utils {
    pub mod gadgets {
        pub mod regex;
        pub mod stopwatch;
    }
    pub mod server_init {
        pub mod cache_load_funcs {}
        pub mod server_init_funcs {
            pub mod https_redirector;
            pub mod initialize_crypto;
            pub mod initialize_logger;
            pub mod load_cert_config;
            pub mod load_env_vars;
        }
        pub mod initialize_server;
        pub mod server_state_def;
    }
}

use chrono::{DateTime, Utc};
use utils::{
    gadgets::stopwatch::Stopwatch,
    server_init::{
        initialize_server::init_server,
        server_init_funcs::{initialize_logger::init_logger, load_env_vars::load_env},
    },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let server_start_time: DateTime<Utc> = Utc::now();
    let mut stopwatch: Stopwatch = Stopwatch::new("cyhdev.com backend server starting...");

    // initialize logger
    init_logger()?;
    stopwatch.click("logging inititalized");

    // load .env files
    let env_path = load_env()?;
    stopwatch.click(&format!("environment variables loaded from {:?}", env_path));

    init_server(&mut stopwatch, server_start_time).await?;

    Ok(())
}
