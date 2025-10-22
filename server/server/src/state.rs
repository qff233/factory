use std::{collections::HashMap, sync::Arc};

use sqlx::PgPool;
use tokio::sync::RwLock;

pub type TokenStore = Arc<RwLock<HashMap<String, i32>>>;

#[derive(Debug, Clone)]
pub struct AppState {
    pub token_store: TokenStore,
    pub db_pool: PgPool,
}

impl AppState {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            token_store: Arc::new(RwLock::new(HashMap::new())),
            db_pool,
        }
    }
}
