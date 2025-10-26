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
}
