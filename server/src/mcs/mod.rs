use std::rc::Rc;

pub mod prelude;
mod queue;
mod repository;
mod track;
mod vehicle;

use prelude::*;

struct MCS {
    sql: Rc<sqlx::PgPool>,
}
