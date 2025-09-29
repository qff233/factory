use std::sync::Arc;

use sqlx::PgPool;

pub struct DbClient {
    pool: Arc<PgPool>,
}

impl DbClient {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn transport(&self) {
        todo!()
    }

    fn insert_transport(&self) {
        todo!()
    }

    fn insert_stocker(&self) {
        todo!()
    }

    fn stocker(&self) {
        todo!()
    }
}
