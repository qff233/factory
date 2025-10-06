use std::sync::Arc;

use sqlx::query;

use crate::db_manager::DbManager;
use crate::transport::schedule::{Error, Result};
use crate::transport::vehicle::ToolType;

#[derive(Debug)]
pub struct ScheduleAdder {
    db: Arc<DbManager>,
}

impl ScheduleAdder {
    pub fn new(db: Arc<DbManager>) -> Self {
        Self { db }
    }

    pub async fn trans_items(&mut self, from: &str, to: &str) -> Result<()> {
        let mut conn = self.db.transport().await.map_err(Error::Db)?;
        query("INSERT INTO item(begin_node_name, end_node_name) VALUES($1,$2)")
            .bind(from)
            .bind(to)
            .execute(&mut *conn)
            .await
            .map_err(Error::Db)?;
        Ok(())
    }

    pub async fn trans_fluid(&mut self, from: &str, to: &str) -> Result<()> {
        let mut conn = self.db.transport().await.map_err(Error::Db)?;
        query("INSERT INTO fluid(begin_node_name, end_node_name) VALUES($1,$2)")
            .bind(from)
            .bind(to)
            .execute(&mut *conn)
            .await
            .map_err(Error::Db)?;
        Ok(())
    }

    pub async fn use_tool(&mut self, pos: &str, tool_type: ToolType) -> Result<()> {
        let mut conn = self.db.transport().await.map_err(Error::Db)?;
        query("INSERT INTO use_tool(end_node_name, tool_type) VALUES($1,$2)")
            .bind(pos)
            .bind(tool_type)
            .execute(&mut *conn)
            .await
            .map_err(Error::Db)?;
        Ok(())
    }
}
