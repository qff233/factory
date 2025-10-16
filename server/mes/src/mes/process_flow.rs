use std::collections::LinkedList;

pub struct Task {
    pub recipe_name: String,
    pub run_count: u32,
}

pub struct ProcessFlow {
    name: String,
    task_sequence: LinkedList<Task>,
    priority: u32,
}

impl ProcessFlow {
    pub fn new(name: &str, task_sequence: impl Into<LinkedList<Task>>, priority: u32) -> Self {
        let name = name.to_string();
        let task_sequence = task_sequence.into();
        Self {
            name,
            task_sequence,
            priority,
        }
    }

    pub async fn save_to_db(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        todo!()
    }
}
