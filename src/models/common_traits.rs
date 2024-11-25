pub trait FromRow: Sized {
    fn from_row(row: tokio_postgres::Row) -> Self;
}

pub trait FromRows: Sized {
    fn from_rows(rows: Vec<tokio_postgres::Row>) -> Vec<Self>;
}

pub trait ToInsertStmt {
    fn to_insert_stmt() -> String;
}

pub trait ToBatchInsertStmt {
    fn to_batch_insert_stmt() -> String;
}
