use chrono::{DateTime, Utc};
use deadpool_postgres::{Object, Transaction};
use uuid::Uuid;

use super::common_traits::{FromRow, FromRows, ToInsertStmt};

pub struct UserToken {
    user_token_id: Uuid,                  // Token's primary key.
    user_token_user_id: Uuid,             // Reference to the user this token belongs to.
    user_token_type: String,              // The type of the token, indicating its purpose.
    user_token_value: String,             // The secure, unique value of the token.
    user_token_created_at: DateTime<Utc>, // The time when the token was generated.
    user_token_expires_at: DateTime<Utc>, // The time when the token will expire and become invalid.
    user_token_used: bool,                // Indicates whether the token has been used or redeemed.
}

impl FromRow for UserToken {
    fn from_row(row: tokio_postgres::Row) -> UserToken {
        UserToken {
            user_token_id: row.get::<&str, Uuid>("user_token_id"),
            user_token_user_id: row.get::<&str, Uuid>("user_token_user_id"),
            user_token_type: row.get::<&str, String>("user_token_type"),
            user_token_value: row.get::<&str, String>("user_token_value"),
            user_token_created_at: row.get::<&str, DateTime<Utc>>("user_token_created_at"),
            user_token_expires_at: row.get::<&str, DateTime<Utc>>("user_token_expires_at"),
            user_token_used: row.get::<&str, bool>("user_token_used"),
        }
    }
}

impl FromRows for UserToken {
    fn from_rows(rows: Vec<tokio_postgres::Row>) -> Vec<Self> {
        rows.into_iter().map(UserToken::from_row).collect()
    }
}

impl UserToken {
    pub async fn get_by_id(conn: &Object, user_token_id: Uuid) -> anyhow::Result<Option<Self>> {
        match conn
            .query_opt(
                "SELECT * FROM v1.user_tokens WHERE user_token_id = $1",
                &[&user_token_id],
            )
            .await
        {
            Ok(Some(row)) => Ok(Some(UserToken::from_row(row))),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    pub async fn delete_by_id(conn: &Transaction<'_>, user_token_id: Uuid) -> anyhow::Result<u64> {
        let query = "DELETE FROM v1.user_tokens WHERE user_token_id = $1";
        let result = conn.execute(query, &[&user_token_id]).await;

        match result {
            Ok(count) => {
                if count == 0 {
                    Err(anyhow::Error::msg(
                        "Delete operation failed: No matching user token found.",
                    ))
                } else {
                    Ok(count)
                }
            }
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }
}

pub struct UserTokenForm {
    pub user_token_user_id: Uuid,
    pub user_token_type: String,
    pub user_token_value: String,
    pub user_token_expires_at: DateTime<Utc>,
}

impl ToInsertStmt for UserTokenForm {
    fn to_insert_stmt() -> String {
        String::from(
            "INSERT INTO v1.user_tokens (user_token_user_id, user_token_type, user_token_value, user_token_created_at, user_token_expires_at, user_token_used) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
    }
}

impl UserTokenForm {
    pub async fn insert(&self, conn: &Transaction<'_>) -> anyhow::Result<UserToken> {
        let now = Utc::now();
        let user_token_used = false;
        match conn
            .query_one(
                &UserTokenForm::to_insert_stmt(),
                &[
                    &self.user_token_user_id,
                    &self.user_token_type,
                    &self.user_token_value,
                    &now,
                    &self.user_token_expires_at,
                    &user_token_used,
                ],
            )
            .await
        {
            Ok(row) => Ok(UserToken::from_row(row)),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }
}

pub struct UserTokenUpdateForm {
    pub user_token_type: Option<String>,
    pub user_token_value: Option<String>,
    pub user_token_expires_at: Option<DateTime<Utc>>,
    pub user_token_used: Option<bool>,
}

impl UserTokenUpdateForm {
    pub async fn update_db(
        &self,
        conn: &Transaction<'_>,
        user_token_id: Uuid,
    ) -> anyhow::Result<Option<UserToken>> {
        let mut set_clauses = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        let mut idx = 1;

        if let Some(ref token_type) = self.user_token_type {
            set_clauses.push(format!("user_token_type = ${}", idx));
            params.push(token_type);
            idx += 1;
        }
        if let Some(ref token_value) = self.user_token_value {
            set_clauses.push(format!("user_token_value = ${}", idx));
            params.push(token_value);
            idx += 1;
        }
        if let Some(ref expires_at) = self.user_token_expires_at {
            set_clauses.push(format!("user_token_expires_at = ${}", idx));
            params.push(expires_at);
            idx += 1;
        }
        if let Some(ref used) = self.user_token_used {
            set_clauses.push(format!("user_token_used = ${}", idx));
            params.push(used);
            idx += 1;
        }

        if set_clauses.is_empty() {
            return Ok(None); // Nothing to update
        }

        let set_clause = set_clauses.join(", ");

        let query = format!(
            "UPDATE v1.user_tokens SET {} WHERE user_token_id = ${} RETURNING *",
            set_clause,
            idx
        );
        params.push(&user_token_id);

        let result = conn.query_opt(&query, &params).await;

        match result {
            Ok(opt_row) => Ok(opt_row.map(UserToken::from_row)),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }
}
