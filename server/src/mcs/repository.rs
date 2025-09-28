use std::rc::Rc;

use sqlx::PgPool;

pub struct Repository {
    pool: Rc<PgPool>,
}

impl Repository {
    pub fn new(pool: Rc<PgPool>) -> Self {
        Self { pool }
    }

    pub fn transport(&self) {
        todo!()
    }

    pub fn insert_transport(&self) {
        todo!()
    }

    pub fn insert_stocker(&self) {
        todo!()
    }

    pub fn stocker(&self) {
        todo!()
    }
}
