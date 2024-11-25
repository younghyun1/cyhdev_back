use chrono::{DateTime, Utc};
use deadpool_postgres::Transaction;
use tokio_postgres::types::Type;

use crate::utils::gadgets::argon::hash_password;

use super::common_traits::{FromRow, FromRows, ToBatchInsertStmt, ToInsertStmt};

pub struct User {
    user_id: uuid::Uuid,                   // User's PKEY.
    user_screen_name: String,              // User's screen name. Unique.
    user_email: String,                    // User's email. Server-side checked, unique.
    user_password_hash: String,            // User's password hash.
    user_created_at: DateTime<Utc>,        // User's creation time.
    user_recorded_to_db_at: DateTime<Utc>, // The time when the row was persisted to DB.
    user_updated_at: DateTime<Utc>,        // User's update time.
    user_is_active: bool,                  // User's active state.
    user_email_verified: bool,             // User's email verified status.
}

impl FromRow for User {
    fn from_row(row: tokio_postgres::Row) -> User {
        User {
            user_id: row.get::<&str, uuid::Uuid>("user_id"),
            user_screen_name: row.get::<&str, String>("user_screen_name"),
            user_email: row.get::<&str, String>("user_email"),
            user_password_hash: row.get::<&str, String>("user_password_hash"),
            user_created_at: row.get::<&str, DateTime<Utc>>("user_created_at"),
            user_recorded_to_db_at: row.get::<&str, DateTime<Utc>>("user_recorded_to_db_at"),
            user_updated_at: row.get::<&str, DateTime<Utc>>("user_updated_at"),
            user_is_active: row.get::<&str, bool>("user_is_active"),
            user_email_verified: row.get::<&str, bool>("user_email_verified"),
        }
    }
}

impl FromRows for User {
    fn from_rows(rows: Vec<tokio_postgres::Row>) -> Vec<Self> {
        rows.into_iter().map(User::from_row).collect()
    }
}

pub struct UserForm {
    pub user_screen_name: String,
    pub user_email: String,
    pub user_password: String,
}

impl ToInsertStmt for UserForm {
    fn to_insert_stmt() -> String {
        String::from("INSERT INTO v1.users (user_screen_name, user_email, user_password_hash, user_created_at, user_recorded_to_db_at, user_updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
    }
}

impl ToBatchInsertStmt for UserForm {
    fn to_batch_insert_stmt() -> String {
        String::from(
            "INSERT INTO v1.users (
                user_screen_name,
                user_email,
                user_password_hash,
                user_created_at,
                user_recorded_to_db_at,
                user_updated_at
            )
            SELECT
                t1.user_screen_name,
                t1.user_email,
                t1.user_password_hash,
                $4, $5, $6
            FROM
                unnest($1::text[], $2::text[], $3::text[]) AS t1(
                    user_screen_name,
                    user_email,
                    user_password_hash
                )
            RETURNING *;",
        )
    }
}

impl UserForm {
    pub async fn insert(&self, conn: &Transaction<'_>) -> anyhow::Result<User> {
        let now = Utc::now();
        match conn
            .query_one(
                &UserForm::to_insert_stmt(),
                &[
                    &self.user_screen_name,
                    &self.user_email,
                    &hash_password(&self.user_password),
                    &now,
                    &now,
                    &now,
                ],
            )
            .await
        {
            Ok(row) => Ok(User::from_row(row)),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    pub async fn batch_insert(
        batch: Vec<Self>,
        conn: &Transaction<'_>,
    ) -> anyhow::Result<Vec<User>> {
        let now = Utc::now();
        match conn
            .query_typed(
                &UserForm::to_batch_insert_stmt(),
                &[
                    (
                        &batch
                            .iter()
                            .map(|form| &form.user_screen_name)
                            .collect::<Vec<&String>>(),
                        Type::VARCHAR_ARRAY,
                    ),
                    (
                        &batch
                            .iter()
                            .map(|form| &form.user_email)
                            .collect::<Vec<&String>>(),
                        Type::VARCHAR_ARRAY,
                    ),
                    (
                        &batch
                            .iter()
                            .map(|form| hash_password(&form.user_password))
                            .collect::<Vec<String>>(),
                        Type::VARCHAR_ARRAY,
                    ),
                    (&now, Type::TIMESTAMPTZ),
                    (&now, Type::TIMESTAMPTZ),
                    (&now, Type::TIMESTAMPTZ),
                ],
            )
            .await
        {
            Ok(rows) => Ok(User::from_rows(rows)),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }
}
