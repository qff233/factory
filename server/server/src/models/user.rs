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
    // pub async fn create(
    //     pool: &sqlx::PgPool,
    //     username: &str,
    //     password_hash: &str,
    //     role: Option<&str>,
    // ) -> Result<Self, sqlx::Error> {
    //     let role = role.unwrap_or("operator");
    //     let user = sqlx::query_as!(
    //         User,
    //         r#"
    //     INSERT INTO auth.users(username, password_hash, role)
    //     VALUES ($1, $2, $3)
    //     RETURNING *
    //     "#,
    //         username,
    //         password_hash,
    //         role
    //     )
    //     .fetch_one(pool)
    //     .await?;

    //     Ok(user)
    // }

    pub async fn from_username(
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

    pub async fn from_id(pool: &sqlx::PgPool, user_id: i32) -> Result<Option<Self>, sqlx::Error> {
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

    // pub async fn get_permissions(&self, pool: &sqlx::PgPool) -> Result<Vec<String>, sqlx::Error> {
    //     let permissions = sqlx::query!(
    //         r#"
    //         SELECT p.name
    //         FROM auth.user_permissions up
    //         JOIN auth.permissions p ON up.permission_id = p.id
    //         WHERE up.user_id = $1
    //         "#,
    //         self.id
    //     )
    //     .fetch_all(pool)
    //     .await?;

    //     Ok(permissions.into_iter().map(|p| p.name).collect())
    // }

    // pub async fn has_permission(&self, pool: &sqlx::PgPool, permission: &str) -> bool {
    //     let result = sqlx::query!(
    //         r#"
    //         SELECT EXISTS(
    //             SELECT 1
    //             FROM auth.user_permissions up
    //             JOIN auth.permissions p ON up.permission_id = p.id
    //             WHERE up.user_id = $1 AND p.name = $2
    //         ) as exists
    //         "#,
    //         self.id,
    //         permission
    //     )
    //     .fetch_optional(pool)
    //     .await;

    //     matches!(result, Ok(Some(_)))
    // }
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
    async fn from_id() {
        let pool = get_pool().await;
        let user = User::from_id(&pool, 1).await;
        println!("{:?}", user);
    }

    #[tokio::test]
    async fn from_username() {
        let pool = get_pool().await;
        let user = User::from_username(&pool, "qff233").await;
        println!("{:?}", user);
    }
}
