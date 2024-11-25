use tracing_subscriber::EnvFilter;

/// initialize logger using 'tracing' crate
pub fn init_logger() -> anyhow::Result<()> {
    // adjust logging levels here
    let mut filter: EnvFilter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?; //for prod

    // exclude output from external crates here
    filter = filter
        .add_directive("tower_http=debug".parse()?)
        .add_directive("axum-template=info".parse()?)
        .add_directive("rustls=off".parse()?)
        // .add_directive("tokio_postgres=debug".parse()?)
        .add_directive("aws_config=off".parse()?);

    tracing_subscriber::fmt()
        .with_ansi(false) // disable colored output; advisable if persisting logs to external
        .with_target(false) // disable target display
        .with_env_filter(filter)
        // .with_writer(non_blocking) // output to a log file
        .init();

    Ok(())
}
