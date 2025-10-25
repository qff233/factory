use std::{collections::HashMap, sync::Arc};

use chrono::Local;
use sqlx::PgPool;
use tokio::sync::RwLock;

pub type TokenStore = Arc<RwLock<HashMap<String, Arc<User>>>>;

#[derive(Debug)]
pub struct User {
    pub user_id: i32,
    pub update_timestamp: RwLock<chrono::DateTime<Local>>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub token_store: TokenStore,
    pub db_pool: PgPool,
}

impl AppState {
    pub async fn new(db_pool: PgPool) -> Self {
        let token_store: TokenStore = Arc::new(RwLock::new(HashMap::new()));

        let inner_token_store = token_store.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;

                {
                    let token_store = inner_token_store.read().await;
                    let now = Local::now();

                    let mut token_to_remove = Vec::new();
                    for (token, user) in token_store.iter() {
                        let duration = now - *user.update_timestamp.read().await;
                        if duration.num_minutes() > 5 {
                            token_to_remove.push(token.clone());
                        }
                    }

                    if !token_to_remove.is_empty() {
                        drop(token_store);
                        let mut token_store = inner_token_store.write().await;
                        for token in token_to_remove {
                            token_store.remove(&token);
                        }
                    }
                };
            }
        });

        Self {
            token_store,
            db_pool,
        }
    }
}
