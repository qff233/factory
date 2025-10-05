use std::sync::Arc;

use sqlx::{PgPool, Postgres, pool::PoolConnection, query};

#[derive(Debug)]
pub struct DbManager {
    pool: PgPool,
}

impl DbManager {
    pub fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { pool })
    }

    pub async fn track(&self) -> Result<PoolConnection<Postgres>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        query("SET search_path TO track;")
            .execute(&mut *conn)
            .await
            .unwrap();
        Ok(conn)
    }

    pub async fn transport(&self) -> Result<PoolConnection<Postgres>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        query("SET search_path TO transport;")
            .execute(&mut *conn)
            .await
            .unwrap();
        Ok(conn)
    }
}
