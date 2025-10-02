use std::sync::Arc;

use sqlx::PgPool;

use crate::transport::vehicle::ToolType;

pub struct DbClient {
    pool: Arc<PgPool>,
}

enum State {
    Pending,
    Transporting,
    Completed,
}

struct ItemTask {
    id: u32,
    start_node_name: String,
    end_node_name: String,
    state: State,
}

struct FluidTask {
    id: u32,
    start_node_name: String,
    end_node_name: String,
    state: State,
}

struct UseToolTask {
    id: u32,
    end_node_name: String,
    tool_type: ToolType,
    state: State,
}

impl DbClient {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_tasks() {
        
    }
}
