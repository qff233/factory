use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub struct Process {
    pub id: i32,
    pub name: String,
    pub step: i32,
    pub tool_id: String,
    pub recipe_name: String,
    pub quantity: i32,
    pub step_description: Option<String>,
    pub created_by: String,
    pub created_at: chrono::NaiveDateTime,
}

impl Process {
    pub async fn update(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        todo!()
    }
}

#[derive(Debug, Serialize)]
pub struct Step {
    pub processes: Vec<Process>,
}

#[derive(Debug, Serialize)]
pub struct ProcessFlow {
    pub name: String,
    pub steps: Vec<Step>,
}

impl ProcessFlow {
    pub async fn from_name(pool: &PgPool, name: &str) -> Option<Self> {
        todo!()
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn from_name() {}

    async fn update() {}
}
