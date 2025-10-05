use std::{collections::HashMap, sync::Arc};

use sqlx::{PgConnection, prelude::FromRow};
use tokio::{
    sync::RwLock,
    time::{self},
};
use tracing::error;

use crate::{
    constant,
    transport::{
        db_manager::DbManager,
        schedule::{Error, Result},
        track::{self, Graph},
        vehicle::{ActionSequence, ActionSequenceBuilder, Skill, ToolType, Vehicle},
    },
};

#[derive(Debug, FromRow)]
struct ItemFluidRow {
    id: i32,
    begin_node_name: String,
    end_node_name: String,
}

#[derive(Debug, FromRow)]
struct UseToolRow {
    id: i32,
    end_node_name: String,
    tool_type: ToolType,
}

#[derive(Debug)]
pub struct ActionPlanner {
    vehicles: Arc<RwLock<HashMap<i32, Vehicle>>>,
    track_graph: Arc<Graph>,
    db: Arc<DbManager>,
}

impl ActionPlanner {
    pub fn run(
        vehicles: Arc<RwLock<HashMap<i32, Vehicle>>>,
        track_graph: Arc<Graph>,
        db: Arc<DbManager>,
    ) {
        let planner = Self {
            vehicles,
            track_graph,
            db,
        };
        tokio::spawn(async move { planner.task().await });
    }

    async fn task(mut self) {
        let mut interval =
            time::interval(time::Duration::from_secs(constant::VEHICLE_SCHEDULE_TIME));
        loop {
            interval.tick().await;
            if let Err(e) = self.plan().await {
                if let Error::VehicleBusy = e {
                } else {
                    error!("ActionPlanner suffer error: {:#?}", e);
                }
            }
        }
    }

    async fn find_idle_vehicle_shortest_path_by_skill(
        &self,
        to: &str,
        skill: Skill,
    ) -> Option<(i32, track::Path)> {
        let mut result: Vec<(i32, track::Path)> = Vec::new();
        for (id, vehicle) in self.vehicles.read().await.iter() {
            if skill != *vehicle.skill() || !vehicle.idle().await {
                continue;
            }

            if let Ok(path) = self.track_graph.find_path(&vehicle.node().map_err(|e|{
                error!("vehicle({}): current node not find in idle. may be not in trackgraph or dont init. error type is {:?}.", {id}, {e});
            }).ok()?.name, to).await {
                result.push((*id, path));
            }
        }
        result.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        result.first().cloned()
    }

    async fn trans_item_actions(
        &self,
        begin_node_name: &str,
        end_node_name: &str,
    ) -> Result<(i32, ActionSequence)> {
        let (id, to_begin_path) = self
            .find_idle_vehicle_shortest_path_by_skill(begin_node_name, Skill::Item)
            .await
            .ok_or(Error::VehicleBusy)?;
        let begin_to_end_path = self
            .track_graph
            .find_path(begin_node_name, end_node_name)
            .await
            .map_err(Error::Db)?;
        Ok((
            id,
            ActionSequenceBuilder::new()
                .move_path(&to_begin_path)
                .suck()
                .move_path(&begin_to_end_path)
                .drop()
                .build(),
        ))
    }

    async fn trans_fluid_actions(
        &self,
        begin_node_name: &str,
        end_node_name: &str,
    ) -> Result<(i32, ActionSequence)> {
        let (id, to_begin_path) = self
            .find_idle_vehicle_shortest_path_by_skill(begin_node_name, Skill::Fluid)
            .await
            .ok_or(Error::VehicleBusy)?;

        let begin_to_end_path = self
            .track_graph
            .find_path(begin_node_name, end_node_name)
            .await
            .map_err(Error::Db)?;

        let to_shipping_dock_path = self
            .track_graph
            .find_shipping_dock_path(end_node_name)
            .await
            .map_err(Error::Db)?;

        Ok((
            id,
            ActionSequenceBuilder::new()
                .move_path(&to_begin_path)
                .suck()
                .move_path(&begin_to_end_path)
                .fill()
                .move_path(&to_shipping_dock_path)
                .drop()
                .build(),
        ))
    }

    async fn use_tool_actions(
        &self,
        node_name: &str,
        tool_type: ToolType,
    ) -> Result<(i32, ActionSequence)> {
        let (id, to_end_path) = self
            .find_idle_vehicle_shortest_path_by_skill(node_name, Skill::UseTool(tool_type))
            .await
            .ok_or(Error::VehicleBusy)?;

        Ok((
            id,
            ActionSequenceBuilder::new()
                .move_path(&to_end_path)
                .use_tool()
                .build(),
        ))
    }

    async fn update_state_processing(
        &self,
        conn: &mut PgConnection,
        table_name: &str,
        vehicle_id: i32,
        task_id: i32,
    ) -> Result<()> {
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
        Ok(())
    }

    async fn get_item_rows(&self, conn: &mut PgConnection) -> Result<Vec<ItemFluidRow>> {
        sqlx::query_as::<_, ItemFluidRow>(
            "
            SELECT id,begin_node_name,end_node_name
            FROM item
            WHERE state = 'pending'
            LIMIT 20;
        ",
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(Error::Db)
    }

    async fn get_fluid_rows(&self, conn: &mut PgConnection) -> Result<Vec<ItemFluidRow>> {
        sqlx::query_as::<_, ItemFluidRow>(
            "
            SELECT id, begin_node_name, end_node_name
            FROM fluid
            WHERE state = 'pending'
            LIMIT 20;
        ",
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(Error::Db)
    }

    async fn get_use_tool_rows(&self, conn: &mut PgConnection) -> Result<Vec<UseToolRow>> {
        sqlx::query_as::<_, UseToolRow>(
            "
            SELECT id, end_node_name, tool_type
            FROM use_tool
            WHERE state = 'pending'
            LIMIT 20;
        ",
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(Error::Db)
    }

    async fn plan_tran_item(
        &self,
        item_rows: Vec<ItemFluidRow>,
        conn: &mut PgConnection,
    ) -> Result<()> {
        for row in item_rows {
            let (vehicle_id, actions) = self
                .trans_item_actions(&row.begin_node_name.trim(), &row.end_node_name.trim())
                .await?;

            self.vehicles
                .write()
                .await
                .get_mut(&vehicle_id)
                .ok_or(Error::VehicleBusy)?
                .processing(actions)
                .await
                .map_err(|_| Error::VehicleBusy)?;
            self.update_state_processing(conn, "item", vehicle_id, row.id)
                .await?;
        }
        Ok(())
    }

    async fn plan_tran_fluid(
        &self,
        item_rows: Vec<ItemFluidRow>,
        conn: &mut PgConnection,
    ) -> Result<()> {
        for row in item_rows {
            let (vehicle_id, actions) = self
                .trans_fluid_actions(&row.begin_node_name.trim(), &row.end_node_name.trim())
                .await?;

            self.vehicles
                .write()
                .await
                .get_mut(&vehicle_id)
                .ok_or(Error::VehicleBusy)?
                .processing(actions)
                .await
                .map_err(|_| Error::VehicleBusy)?;
            self.update_state_processing(&mut *conn, "fluid", vehicle_id, row.id)
                .await?;
        }
        Ok(())
    }

    async fn plan_use_tool(
        &self,
        item_rows: Vec<UseToolRow>,
        conn: &mut PgConnection,
    ) -> Result<()> {
        for row in item_rows {
            let (vehicle_id, actions) = self
                .use_tool_actions(&row.end_node_name.trim(), row.tool_type)
                .await?;

            self.vehicles
                .write()
                .await
                .get_mut(&vehicle_id)
                .ok_or(Error::VehicleBusy)?
                .processing(actions)
                .await
                .map_err(|_| Error::VehicleBusy)?;
            self.update_state_processing(&mut *conn, "use_tool", vehicle_id, row.id)
                .await?;
        }
        Ok(())
    }

    async fn plan(&mut self) -> Result<()> {
        let mut conn = self.db.transport().await.map_err(Error::Db)?;
        let item_rows = self.get_item_rows(&mut *conn).await?;
        if let Err(e) = self.plan_tran_item(item_rows, &mut *conn).await {
            if let Error::VehicleBusy = e {
            } else {
                error!("plan() suffer error: {:#?}", e);
            }
        }

        let fluid_rows = self.get_fluid_rows(&mut *conn).await?;
        if let Err(e) = self.plan_tran_fluid(fluid_rows, &mut *conn).await {
            if let Error::VehicleBusy = e {
            } else {
                error!("plan() suffer error: {:#?}", e);
            }
        }

        let use_tool_rows = self.get_use_tool_rows(&mut *conn).await?;
        if let Err(e) = self.plan_use_tool(use_tool_rows, &mut *conn).await {
            if let Error::VehicleBusy = e {
            } else {
                error!("plan() suffer error: {:#?}", e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use dotenvy::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use tokio::sync::RwLock;

    use crate::transport::{
        db_manager::DbManager, schedule::action_planner::ActionPlanner, track::Graph,
        vehicle::Vehicle,
    };

    #[tokio::test]
    async fn get_rows() {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        println!("Connecting to database: {}", database_url);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool");

        let db = DbManager::new(pool);
        let vehicles = Arc::new(RwLock::new(HashMap::new()));
        let track_graph = Arc::new(Graph::new(db.clone()).await);
        {
            vehicles
                .write()
                .await
                .insert(2500, Vehicle::new(2500, track_graph.clone()).await);
        }

        // let _a = sqlx::query_as::<_, ItemFluidRow>(
        //     "
        //     SELECT id,begin_node_name,end_node_name
        //     FROM item
        //     WHERE state = 'pending'
        //     LIMIT 20;
        // ",
        // )
        // .fetch_all(&mut *db.transport().await.unwrap())
        // .await
        // .unwrap();

        let mut action_planner = ActionPlanner {
            db,
            vehicles,
            track_graph,
        };

        action_planner.plan().await.unwrap();
    }
}
