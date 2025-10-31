use sqlx::PgPool;

enum Status {
    Queue,
    Hold,
    Running,
    Completed,
}

struct Task {
    id: i64,
    product_id: i64,
    recipe_name: String,
    quantity: i32,
}

impl Task {
    pub async fn new_from_tool_id(pool: PgPool, tool_id: &str) -> Option<Self> {
        todo!()
    }

    pub async fn running(pool: PgPool, id: i64) -> Result<(), sqlx::Error> {
        todo!()
    }

    pub async fn completed(pool: PgPool, id: i64) -> Result<(), sqlx::Error> {
        todo!()
    }
}
