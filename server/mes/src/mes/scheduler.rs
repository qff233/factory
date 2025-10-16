use crate::mes::process_flow::Task;

pub struct ProcessFlowOperator {
    pool: sqlx::PgPool,
}

impl ProcessFlowOperator {
    fn pop(&self) -> Option<Task> {
        todo!()
    }
}

pub struct Scheduler {
    pool: sqlx::PgPool,
}

impl Scheduler {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_process_flow() -> Option<ProcessFlowOperator> {
        todo!()
    }

    pub async fn add_process_flow() -> Result<(), sqlx::Error> {
        todo!()
    }
}
