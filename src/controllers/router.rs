use std::sync::Arc;

use axum::middleware::from_fn;
use tower_http::compression::CompressionLayer;

use crate::{models::users::UserForm, utils::server_init::server_state_def::ServerState};

use super::middleware::request_response_info::print_request_info;

pub async fn generate_router(state: &Arc<ServerState>) -> axum::Router {
    let test_batch_insert = async {
        let user1 = UserForm {
            user_screen_name: String::from("user1"),
            user_email: String::from("user1@example.com"),
            user_password: String::from("password123"),
        };

        let user2 = UserForm {
            user_screen_name: String::from("user2"),
            user_email: String::from("user2@example.com"),
            user_password: String::from("password456"),
        };

        let user3 = UserForm {
            user_screen_name: String::from("user3"),
            user_email: String::from("user3@example.com"),
            user_password: String::from("password789"),
        };

        let batch = vec![user1, user2, user3];

        if let Ok(mut conn) = state.get_conn().await {
            if let Ok(transaction) = conn.transaction().await {
                match UserForm::batch_insert(batch, &transaction).await {
                    Ok(users) => {
                        println!("Batch insert successful for {} users", users.len());
                        if transaction.commit().await.is_err() {
                            println!("Error committing transaction");
                        }
                    }
                    Err(e) => {
                        println!("Batch insert failed: {:?}", e);
                    }
                }
            }
        } else {
            println!("Error obtaining transaction");
        }
    };

    test_batch_insert.await;

    axum::Router::new()
        .layer(CompressionLayer::new())
        .layer(from_fn(print_request_info))
        .with_state(Arc::clone(state))
}
