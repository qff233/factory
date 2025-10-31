use std::ops::{Deref, DerefMut};

use serde::Serialize;
use sqlx::{PgPool, prelude::FromRow};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Process {
    pub id: i32,
    pub name: String,
    pub sequence: i32,
    pub tool_id: String,
    pub recipe_name: String,
    pub quantity: i32,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub processes: Vec<Process>,
}

impl Step {
    fn new() -> Self {
        Self {
            processes: Vec::new(),
        }
    }
}
impl Deref for Step {
    type Target = Vec<Process>;

    fn deref(&self) -> &Self::Target {
        &self.processes
    }
}

impl DerefMut for Step {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.processes
    }
}

#[derive(Debug, Serialize)]
pub struct ProcessFlow {
    pub steps: Vec<Step>,
}

impl ProcessFlow {
    pub async fn fetch_from_name(pool: &PgPool, name: &str) -> Option<Self> {
        let processes = sqlx::query_as::<_, Process>(
            r#"
            SELECT * FROM mes.process_flows WHERE name = $1 ORDER BY sequence
            "#,
        )
        .bind(name)
        .fetch_all(pool)
        .await
        .ok()?;

        let mut steps = Vec::new();
        let mut processes_in_step: Step = Step::new();
        let mut last_sequence = processes.first()?.id;
        for process in processes {
            if process.sequence != last_sequence {
                steps.push(processes_in_step.clone());
                processes_in_step.clear();
                last_sequence = process.sequence
            }
            processes_in_step.push(process);
        }
        steps.push(processes_in_step);

        Some(Self { steps })
    }
}

#[derive(Debug, Serialize)]
pub struct ProcessFlowNames {
    pub names: Vec<String>,
}

impl ProcessFlowNames {
    pub async fn fetch_all(pool: &PgPool, limit: i32, offset: i32) -> Option<Self> {
        let names = sqlx::query_scalar(
            r#"
            SELECT name FROM mes.process_flows GROUP BY name LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .ok()?;

        Some(Self { names })
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
    async fn fetch_from_name() {
        let pool = get_pool().await;
        let process_flow = ProcessFlow::fetch_from_name(&pool, "Test Process Flow")
            .await
            .unwrap();
        println!("{:#?}", process_flow);
    }

    #[tokio::test]
    async fn fetch_all_name() {
        let pool = get_pool().await;
        let process_flow_names = ProcessFlowNames::fetch_all(&pool, 200, 0).await.unwrap();
        println!("{:#?}", process_flow_names);
    }
}
