use std::sync::Arc;

use chrono::Local;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::state::{TokenStore, User};

fn generate_token() -> String {
    Uuid::new_v4().to_string()
}

pub async fn validate_token(token_store: &TokenStore, token: &str) -> Option<Arc<User>> {
    let store = token_store.read().await;
    store.get(token).cloned()
}

pub async fn add_token(token_store: &TokenStore, user_id: i32) -> String {
    let token = generate_token();
    let mut store = token_store.write().await;
    store.insert(
        token.clone(),
        Arc::new(User {
            user_id,
            update_timestamp: RwLock::new(Local::now()),
        }),
    );
    token
}

pub async fn remove_token(token_store: &TokenStore, token: &str) {
    let mut store = token_store.write().await;
    store.remove(token);
}
