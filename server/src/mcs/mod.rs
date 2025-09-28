use std::rc::Rc;

pub mod prelude;
mod queue;
mod repository;
mod track;
mod vehicle;

use prelude::*;

use tokio::time;

pub struct MCS {
    sql: Rc<sqlx::PgPool>,
}

impl MCS {
    pub fn new(sql: Rc<sqlx::PgPool>) -> Self {
        Self { sql }
    }

    pub async fn timer_task(&self) {
        let mut interval = time::interval(time::Duration::from_secs(5));
        loop {
            interval.tick().await;
        }
    }
}
