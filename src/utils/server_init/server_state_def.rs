use std::net::{IpAddr, Ipv4Addr};

use chrono::{DateTime, Utc};
use regex::Regex;

use crate::utils::gadgets::{
    regex::{compile_regex, EMAIL_VALIDATION_REGEX},
    stopwatch::Stopwatch,
};

use super::server_init_funcs::load_env_vars::get_env_var;

pub struct ServerState {
    server_config: ServerConfig,
    regexes: CompiledRegexes,
    server_start_time: DateTime<Utc>,
    app_name_version: String,
}

impl ServerState {
    pub fn new(
        stopwatch: &mut Stopwatch,
        server_start_time: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        Ok(ServerState {
            server_config: ServerConfig::new()?,
            regexes: CompiledRegexes::compile()?,
            server_start_time,
            app_name_version: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        })
    }
}

pub struct ServerConfig {
    host_port: u16,
    host_addr: IpAddr,
    smtp_username: String,
    smtp_password: String,
}

impl ServerConfig {
    fn new() -> anyhow::Result<Self> {
        Ok(ServerConfig {
            host_port: get_env_var("HOST_PORT")?.parse::<u16>().unwrap_or(443u16),
            host_addr: get_env_var("HOST_ADDR")?
                .parse::<IpAddr>()
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            smtp_username: get_env_var("SMTP_USERNAME")?,
            smtp_password: get_env_var("SMTP_PASSWORd")?,
        })
    }
}

pub struct CompiledRegexes {
    email_validation_regex: Regex,
}

impl CompiledRegexes {
    fn compile() -> anyhow::Result<Self> {
        Ok(CompiledRegexes {
            email_validation_regex: compile_regex(&EMAIL_VALIDATION_REGEX)?,
        })
    }
}
