use chrono::{DateTime, Utc};
use deadpool_postgres::{Object, Transaction};
use serde_derive::{Deserialize, Serialize};
use tokio_postgres::types::Type;
use uuid::Uuid;

use crate::utils::gadgets::argon::hash_password;

use super::common_traits::{FromRow, FromRows, ToBatchInsertStmt, ToInsertStmt};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct UserTruncated {
    user_id: uuid::Uuid,                   // User's PKEY.
    user_screen_name: String,              // User's screen name. Unique.
    user_email: String,                    // User's email. Server-side checked, unique.
    user_created_at: DateTime<Utc>,        // User's creation time.
}

impl UserTruncated {
    pub fn get_id(&self) -> Uuid {
        self.user_id
    }

    pub fn get_created_at(&self) -> DateTime<Utc> {
        self.user_created_at
    }
}

impl From<User> for UserTruncated {
    fn from(user: User) -> Self {
        UserTruncated {
            user_id: user.user_id,
            user_screen_name: user.user_screen_name,
            user_email: user.user_email,
            user_created_at: user.user_created_at,
        }
    }
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

impl User {
    pub async fn get_by_id(conn: &Object, user_id: Uuid) -> anyhow::Result<Option<Self>> {
        match conn
            .query_opt("SELECT * FROM v1.users WHERE user_id = $1", &[&user_id])
            .await
        {
            Ok(Some(row)) => Ok(Some(User::from_row(row))),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    pub async fn get_by_ids(conn: &Object, user_ids: Vec<Uuid>) -> anyhow::Result<Vec<Self>> {
        let rows = conn
            .query(
                "SELECT * FROM v1.users WHERE user_id = ANY($1)",
                &[&user_ids],
            )
            .await?;
        Ok(User::from_rows(rows))
    }

    pub fn get_id(&self) -> Uuid {
        self.user_id
    }

    pub fn get_created_at(&self) -> DateTime<Utc> {
        self.user_created_at
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
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
    pub async fn insert(&self, conn: &Transaction<'_>) -> Result<User, tokio_postgres::Error> {
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
            Err(e) => Err(e),
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

#[derive(Serialize, Deserialize, Debug)]
pub struct UserUpdateForm {
    pub user_screen_name: Option<String>,
    pub user_email: Option<String>,
    pub user_password: Option<String>,
    pub user_is_active: Option<bool>,
}

impl UserUpdateForm {
    pub async fn update_db(
        &self,
        conn: &Transaction<'_>,
        user_id: Uuid,
    ) -> anyhow::Result<Option<User>> {
        let set_clauses: Vec<_> = [
            self.user_screen_name
                .as_ref()
                .map(|_| "user_screen_name = $1"),
            self.user_email.as_ref().map(|_| "user_email = $2"),
            self.user_password
                .as_ref()
                .map(|_| "user_password_hash = $3"),
            self.user_is_active.map(|_| "user_is_active = $4"),
        ]
        .iter()
        .filter_map(|&x| x)
        .collect();

        if set_clauses.is_empty() {
            return Ok(None); // Nothing to update
        }

        let set_clause = set_clauses.join(", ");

        let query = format!(
            "UPDATE v1.users SET {}, user_updated_at = NOW() WHERE user_id = $5 RETURNING *",
            set_clause
        );

        let result = conn
            .query_opt(
                &query,
                &[
                    &self.user_screen_name,
                    &self.user_email,
                    &self.user_password.as_ref().map(|pwd| hash_password(pwd)),
                    &self.user_is_active,
                    &user_id,
                ],
            )
            .await;

        match result {
            Ok(opt_row) => Ok(opt_row.map(User::from_row)),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }
}

impl User {
    pub async fn delete_by_id(conn: &Transaction<'_>, user_id: Uuid) -> anyhow::Result<u64> {
        let query = "DELETE FROM v1.users WHERE user_id = $1";
        let result = conn.execute(query, &[&user_id]).await;

        match result {
            Ok(count) => {
                if count == 0 {
                    Err(anyhow::Error::msg(
                        "Delete operation failed: No matching user found.",
                    ))
                } else {
                    Ok(count)
                }
            }
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }
}
