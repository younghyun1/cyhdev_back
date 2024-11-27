use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool_postgres::{Object, Pool};
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use regex::Regex;

use crate::utils::gadgets::{
    regex::{compile_regex, EMAIL_VALIDATION_REGEX},
    stopwatch::Stopwatch,
};

use super::server_init_funcs::{
    initialize_db_conn_pool::init_db_conn_pool, initialize_mailer::init_mailer,
    load_env_vars::get_env_var,
};

#[derive(Clone)]
pub struct ServerState {
    cache: Cache,
    server_resources: ServerResources,
}

impl ServerState {
    pub fn new(stopwatch: &mut Stopwatch, server_start_time: DateTime<Utc>) -> Result<Self> {
        Ok(ServerState {
            cache: Cache::new()?,
            server_resources: ServerResources::new(server_start_time)?,
        })
    }
}

impl ServerState {
    pub fn email_regex(&self) -> &Regex {
        &self.server_resources.regexes.email_validation_regex
    }

    pub fn get_name(&self) -> String {
        self.server_resources.app_name_version.clone()
    }

    pub async fn get_conn(&self) -> Result<Object> {
        match self.server_resources.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    pub fn get_request(&self) -> &reqwest::Client {
        &self.server_resources.request_client
    }

    pub fn get_mailer(&self) -> &AsyncSmtpTransport<Tokio1Executor> {
        &self.server_resources.mailer
    }

    pub fn get_socket_addr(&self) -> SocketAddr {
        match self.server_resources.server_config.host_addr {
            IpAddr::V4(ipv4_addr) => SocketAddr::V4(SocketAddrV4::new(
                ipv4_addr,
                self.server_resources.server_config.host_port,
            )),
            IpAddr::V6(ipv6_addr) => SocketAddr::V6(SocketAddrV6::new(
                ipv6_addr,
                self.server_resources.server_config.host_port,
                0,
                0,
            )),
        }
    }
}

#[derive(Clone)]
pub struct Cache {}

impl Cache {
    pub fn new() -> Result<Self> {
        Ok(Cache {})
    }
}

#[derive(Clone)]
pub struct ServerResources {
    server_config: ServerConfig,
    regexes: CompiledRegexes,
    server_start_time: DateTime<Utc>,
    app_name_version: String,
    pool: Pool,
    request_client: reqwest::Client,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl ServerResources {
    pub fn new(server_start_time: DateTime<Utc>) -> anyhow::Result<Self> {
        Ok(ServerResources {
            server_config: ServerConfig::new()?,
            regexes: CompiledRegexes::compile()?,
            server_start_time,
            app_name_version: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            pool: init_db_conn_pool()?,
            request_client: reqwest::Client::new(),
            mailer: init_mailer()?,
        })
    }
}

#[derive(Clone)]
pub struct ServerConfig {
    host_port: u16,
    host_addr: IpAddr,
}

impl ServerConfig {
    pub fn new() -> anyhow::Result<Self> {
        Ok(ServerConfig {
            host_port: get_env_var("HOST_PORT")?.parse::<u16>().unwrap_or(443u16),
            host_addr: get_env_var("HOST_ADDR")?
                .parse::<IpAddr>()
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        })
    }
}

#[derive(Clone)]
pub struct CompiledRegexes {
    email_validation_regex: Regex,
}

impl CompiledRegexes {
    fn compile() -> anyhow::Result<Self> {
        Ok(CompiledRegexes {
            email_validation_regex: compile_regex(EMAIL_VALIDATION_REGEX)?,
        })
    }
}
