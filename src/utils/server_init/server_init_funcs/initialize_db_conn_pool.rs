use deadpool_postgres::{Config, ManagerConfig, Pool};

use tokio_postgres::NoTls;
use tracing::error;

use super::load_env_vars::get_env_var;

pub fn init_db_conn_pool() -> anyhow::Result<Pool> {
    // db configuration from env
    let mut db_config: Config = Config::new();
    db_config.user = Some(get_env_var("DB_USER")?);
    db_config.host = Some(get_env_var("DB_HOST")?);
    db_config.dbname = Some(get_env_var("DB_NAME")?);
    db_config.password = Some(get_env_var("DB_PASSWORD")?);
    db_config.port = Some(get_env_var("DB_PORT")?.parse()?);
    db_config.manager = Some(ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Fast, // look into more later
    });
    let pool: Pool = match db_config.create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls) {
        Ok(pool) => pool,
        Err(e) => {
            error!("Could not create connection pool: {:?}", e);
            return Err(anyhow::anyhow!(e));
        }
    };

    Ok(pool)
}
