use std::rc::Rc;

mod process_flow;
mod scheduler;
mod tool;

struct MesServer {
    sql: Rc<sqlx::PgPool>,
}
