use crate::transport::schedule::{Error, Result};
use crate::transport::vehicle::Skill;
use crate::transport::{db_manager::DbManager, vehicle};
use sqlx::{PgConnection, query};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::error;

pub struct StateUpdate {
    vehicle_event_receiver: mpsc::Receiver<vehicle::Event>,
    db: Arc<DbManager>,
}

impl StateUpdate {
    pub fn run(vehicle_event_receiver: mpsc::Receiver<vehicle::Event>, db: Arc<DbManager>) {
        let status_update = Self {
            vehicle_event_receiver,
            db,
        };
        tokio::spawn(status_update.task());
    }

    async fn task(mut self) {
        while let Some(event) = self.vehicle_event_receiver.recv().await {
            match self.db.transport().await {
                Ok(mut conn) => {
                    if let Err(e) = Self::process_event(&event, &mut *conn).await {
                        error!("Schedule State Update suffer error. {:#?}.", e);
                    }
                }
                Err(e) => {
                    error!("Schedule State Update suffer error. {:#?}.", e);
                }
            }
        }
    }

    async fn process_event(event: &vehicle::Event, conn: &mut PgConnection) -> Result<()> {
        match event {
            vehicle::Event::ProcessDone {
                vehicle_id: _,
                vehicle_skill,
                task_id,
            } => {
                let table_name = match vehicle_skill {
                    Skill::Item => "item",
                    Skill::Fluid => "fluid",
                    Skill::UseTool(_) => "use_tool",
                };
                let query_sql = format!(
                    "
                    UPDATE {}
                    SET state = 'completed'
                    WHERE id = $1;
                ",
                    table_name
                );
                query(&query_sql)
                    .bind(task_id)
                    .execute(conn)
                    .await
                    .map_err(Error::Db)?;
            }
            vehicle::Event::ProcessStart {
                vehicle_id,
                vehicle_skill,
                task_id,
            } => {
                let table_name = match vehicle_skill {
                    Skill::Item => "item",
                    Skill::Fluid => "fluid",
                    Skill::UseTool(_) => "use_tool",
                };
                let query_sql = format!(
                    "
                    UPDATE {}
                    SET vehicle_id = $1,state = 'processing'
                    WHERE id = $2;
                ",
                    table_name
                );
                sqlx::query(&query_sql)
                    .bind(vehicle_id)
                    .bind(task_id)
                    .execute(conn)
                    .await
                    .map_err(Error::Db)?;
            }
            vehicle::Event::ChargeStart | vehicle::Event::ChargeDone => {}
        }

        Ok(())
    }
}
