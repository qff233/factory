use serde::Serialize;
use sqlx::prelude::*;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
}

impl User {
    pub async fn fetch_from_username(
        pool: &sqlx::PgPool,
        username: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM auth.users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn fetch_from_id(
        pool: &sqlx::PgPool,
        user_id: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT id, username, password_hash, role
                FROM auth.users
                WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub fn verify_password(&self, password_hash: &str) -> bool {
        self.password_hash == password_hash
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{PgPool, postgres::PgPoolOptions};

    use super::*;

    async fn get_pool() -> PgPool {
        dotenvy::dotenv().unwrap();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool")
    }

    #[tokio::test]
    async fn fetch_from_id() {
        let pool = get_pool().await;
        let user = User::fetch_from_id(&pool, 1).await;
        println!("{:?}", user);
    }

    #[tokio::test]
    async fn fetch_from_username() {
        let pool = get_pool().await;
        let user = User::fetch_from_username(&pool, "qff233").await;
        println!("{:?}", user);
    }
}
