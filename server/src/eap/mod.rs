use std::rc::Rc;

struct EAPServer {
    sql: Rc<sqlx::PgPool>,
}
