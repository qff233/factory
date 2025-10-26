use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::*};

#[derive(sqlx::Type, Serialize, Debug, Deserialize)]
#[sqlx(type_name = "mes.recipe_status", rename_all = "lowercase")]
pub enum Status {
    Active,
    Inactive,
}

#[derive(FromRow, Serialize, Debug)]
pub struct Recipe {
    id: i32,
    tool_type: String,
    name: String,
    version: String,
    status: Status,
    inputs: Option<Vec<String>>,
    inputbuss: Option<Vec<String>>,
    created_by: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

impl Recipe {
    pub async fn update(
        pool: &PgPool,
        id: i32,
        tool_type: Option<&str>,
        name: Option<&str>,
        version: Option<&str>,
        inputs: Option<&Vec<String>>,
        inputbuss: Option<&Vec<String>>,
    ) -> Result<Self, sqlx::Error> {
        let recipe = sqlx::query_as::<_, Recipe>(
            r#"
            WITH tt AS (
                SELECT id, name 
                FROM mes.tool_types
                WHERE name = $2
            )
            UPDATE mes.recipes
            SET 
                tool_type = COALESCE((SELECT id FROM tt), tool_type),
                name = COALESCE($3, name),
                version = COALESCE($4, version),
                inputs = COALESCE($5, inputs),
                inputbuss = COALESCE($6, inputbuss)
            WHERE id = $1
            RETURNING
                id,
                (SELECT name FROM mes.tool_types WHERE id=tool_type) as tool_type,
                name,
                version,
                status,
                inputs,
                inputbuss,
                created_by,
                created_at,
                updated_at;
            "#,
        )
        .bind(id)
        .bind(tool_type)
        .bind(name)
        .bind(version)
        .bind(inputs)
        .bind(inputbuss)
        .fetch_one(pool)
        .await?;

        Ok(recipe)
    }

    pub async fn active(pool: &PgPool, id: i32) -> Result<String, sqlx::Error> {
        #[derive(FromRow)]
        struct Row {
            active_recipe: Option<String>,
        }

        let message = sqlx::query_as!(Row, "SELECT * FROM mes.active_recipe($1)", id)
            .fetch_one(pool)
            .await?;

        Ok(message.active_recipe.unwrap_or("内部错误".to_string()))
    }
}

#[derive(Debug, Serialize)]
pub struct Recipes(Vec<Recipe>);

impl Recipes {
    pub async fn from_tool_type_and_name(
        pool: &PgPool,
        tool_type: Option<&str>,
        name: Option<&str>,
        page_count: Option<i32>,
        page_index: Option<i32>,
    ) -> Result<Self, sqlx::Error> {
        let tool_type = tool_type.unwrap_or("%");
        let name = name.unwrap_or("%");
        let limit = page_count.unwrap_or(100);
        let offset = page_index.unwrap_or(0);
        let recipes = sqlx::query_as::<_, Recipe>(
            r#"
            SELECT 
                r.id, 
                tt.name as tool_type,
                r.name,
                r.version,
                r.status,
                r.inputs,
                r.inputbuss,
                r.created_by,
                r.created_at,
                r.updated_at
            FROM mes.recipes r
            JOIN mes.tool_types tt ON r.tool_type = tt.id
            WHERE tt.name LIKE $1 AND r.name LIKE $2
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tool_type)
        .bind(name)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok(Self(recipes))
    }
}

#[cfg(test)]
mod tests {
    use sqlx::postgres::PgPoolOptions;

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
    async fn get() {
        let pool = get_pool().await;
        let recipes = Recipes::from_tool_type_and_name(&pool, None, None, None, None)
            .await
            .unwrap();
        println!("{:#?}", recipes);
    }

    #[tokio::test]
    async fn update() {
        let pool = get_pool().await;
        let new_recipe = Recipe::update(&pool, 1, None, None, Some("0.5"), None, None)
            .await
            .unwrap();
        println!("{:#?}", new_recipe);
    }

    #[tokio::test]
    async fn active() {
        let pool = get_pool().await;
        let message = Recipe::active(&pool, 1).await.unwrap();
        println!("{}", message);
        let message = Recipe::active(&pool, 8).await.unwrap_err();
        println!("{}", message.as_database_error().unwrap().message());
    }
}
