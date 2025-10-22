use uuid::Uuid;

use crate::state::TokenStore;

fn generate_token() -> String {
    Uuid::new_v4().to_string()
}

pub async fn validate_token(token_store: &TokenStore, token: &str) -> Option<i32> {
    let store = token_store.read().await;
    store.get(token).cloned()
}

pub async fn add_token(token_store: &TokenStore, user_id: i32) -> String {
    let token = generate_token();
    let mut store = token_store.write().await;
    store.insert(token.clone(), user_id);
    token
}

pub async fn remove_token(token_store: &TokenStore, token: &str) {
    let mut store = token_store.write().await;
    store.remove(token);
}
