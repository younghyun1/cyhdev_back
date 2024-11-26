#[macro_export]
macro_rules! get_conn {
    ($state:expr, $stopwatch:expr) => {{
        match $state.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                return ErrResp::from(ErrRespDat::COULD_NOT_GET_CONN_FROM_POOL, &$stopwatch, e)
                    .into_response();
            }
        }
    }};
}

#[macro_export]
macro_rules! get_transaction {
    ($conn:expr, $stopwatch:expr) => {{
        match $conn.transaction().await {
            Ok(tran) => tran,
            Err(e) => {
                return ErrResp::from(
                    ErrRespDat::COULD_NOT_BUILD_TRANSACTION_FROM_CONN,
                    &$stopwatch,
                    anyhow::Error::from(e),
                )
                .into_response()
            }
        }
    }};
}
