use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, mpsc};
use tracing::warn;

use crate::{
    constant,
    transport::{
        prelude::Position,
        schedule::{Task, TaskList, action_planner::ActionPlanner},
        track::Graph,
        vehicle::{Action, Vehicle},
    },
};

#[derive(Debug)]
pub struct ScheduleExec {
    track_graph: Arc<Graph>,
    vehicles: Arc<RwLock<HashMap<u32, Vehicle>>>,
}

impl ScheduleExec {
    pub async fn new(mut receiver: mpsc::Receiver<Task>, track_graph: Graph) -> Self {
        let vehicles = Arc::new(RwLock::new(HashMap::new()));
        let track_graph = Arc::new(track_graph);
        let pending_tasks = Arc::new(RwLock::new(TaskList::new()));

        let inner_pending_tasks = pending_tasks.clone();
        ActionPlanner::run(vehicles.clone(), track_graph.clone(), pending_tasks);
        tokio::spawn(async move {
            while let Some(task) = receiver.recv().await {
                let mut tasks = inner_pending_tasks.write().await;
                match task {
                    Task::TransItem { .. } => tasks.trans_item_task.push_back(task),
                    Task::TransFluid { .. } => tasks.trans_fluid_task.push_back(task),
                    Task::UseTool { .. } => tasks.use_tool_task.push_back(task),
                }
            }
        });

        Self {
            track_graph,
            vehicles,
        }
    }

    pub async fn get_action(
        &mut self,
        id: u32,
        position: impl Into<Position>,
        battery_level: f32,
        tool_level: Option<f32>,
    ) -> Option<Action> {
        if let Some(tool_level) = tool_level
            && tool_level < constant::VEHICLE_TOOL_WARN_LEVEL
        {
            warn!("{} suffer low tool level", id);
        }
        let position = &position.into();
        let mut vehicles = self.vehicles.write().await;
        match vehicles.get_mut(&id) {
            Some(vehicle) => vehicle.get_action(position, battery_level).await,
            None => {
                let mut vehicle = Vehicle::new(id, self.track_graph.clone()).await;
                let action = vehicle.get_action(position, battery_level).await;
                vehicles.insert(id, vehicle);
                action
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::db_manager::DbManager;
    use dotenvy::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

    async fn get_track_graph() -> Graph {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        println!("Connecting to database: {}", database_url);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool");
        let db_manaer = DbManager::new(pool);
        Graph::new(db_manaer).await
    }

    #[tokio::test]
    async fn dispatch() {
        tracing_subscriber::registry().with(fmt::layer()).init();
        let track_graph = get_track_graph().await;
        let (sender, receiver) = mpsc::channel(200);
        sender
            .send(Task::TransItem {
                begin_node_name: "S2".to_string(),
                end_node_name: "S1".to_string(),
            })
            .await
            .unwrap();

        let mut dispatch = ScheduleExec::new(receiver, track_graph).await;

        // Item

        assert!(
            matches!(dispatch .get_action(2500, (0.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "S3")
        );
        assert!(matches!(
            dispatch
                .get_action(2500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
                .await
                .unwrap(),
            Action::Drop
        ));

        assert!(
            matches!(dispatch .get_action(2500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );

        // Yield to recv next task
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        assert!(
            matches!(dispatch .get_action(2500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );
        println!("{:#?}", dispatch.vehicles.read().await);
        assert!(
            matches!(dispatch .get_action(2500, (0.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A6")
        );
        assert!(
            matches!(dispatch.get_action(2500, (0.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "S2")
        );
        assert!(matches!(
            dispatch
                .get_action(2500, (-1.0, 1.0, 0.0), 1.0, Some(1.0))
                .await
                .unwrap(),
            Action::Suck
        ));
        assert!(
            matches!(dispatch.get_action(2500, (-1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A6")
        );
        assert!(
            matches!(dispatch.get_action(2500, (0.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A2")
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A1")
        );
        assert!(
            matches!(dispatch.get_action(2500, (2.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A4")
        );
        assert!(
            matches!(dispatch.get_action(2500, (2.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A3")
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "S1")
        );
        assert!(matches!(
            dispatch
                .get_action(2500, (1.0, 3.0, 0.0), 1.0, Some(1.0))
                .await
                .unwrap(),
            Action::Drop
        ));
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 3.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A3")
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A2")
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A1")
        );
        assert!(
            matches!(dispatch.get_action(2500, (2.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "P1")
        );
        assert!(
            dispatch
                .get_action(2500, (2.0, 0.0, 0.0), 1.0, Some(1.0))
                .await
                .is_none()
        );

        // println!(
        //     "locked node: {:#?}",
        //     dispatch.track_graph.get_lock_nodes().await
        // );

        // Fluid
        assert!(
            matches!(dispatch.get_action(5500, (2.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A4")
        );
        assert!(
            matches!(dispatch.get_action(5500, (2.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A3")
        );
        assert!(
            matches!(dispatch.get_action(5500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );
        assert!(
            matches!(dispatch.get_action(5500, (0.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "S3")
        );
        assert!(matches!(
            dispatch
                .get_action(5500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
                .await
                .unwrap(),
            Action::Fill
        ));
        assert!(
            matches!(dispatch.get_action(5500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );

        sender
            .send(Task::TransFluid {
                begin_node_name: "S1".to_string(),
                end_node_name: "S2".to_string(),
            })
            .await
            .unwrap();
        // Yield to recv next task
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        assert!(
            matches!(dispatch.get_action(5500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );
        assert!(
            matches!(dispatch.get_action(5500, (0.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A6")
        );

        assert!(
            matches!(dispatch.get_action(5500, (0.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A2")
        );
        assert!(
            matches!(dispatch.get_action(5500, (1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A1")
        );
        assert!(
            matches!(dispatch.get_action(5500, (2.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A4")
        );
        assert!(
            matches!(dispatch .get_action(5500, (2.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A3")
        );
        assert!(
            matches!(dispatch .get_action(5500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "S1")
        );
        assert!(matches!(
            dispatch
                .get_action(5500, (1.0, 3.0, 0.0), 1.0, Some(1.0))
                .await
                .unwrap(),
            Action::Drain
        ));
        assert!(
            matches!(dispatch .get_action(5500, (1.0, 3.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A3")
        );
        assert!(
            matches!(dispatch .get_action(5500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );
        assert!(
            matches!(dispatch .get_action(5500, (0.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A6")
        );
        assert!(
            matches!(dispatch .get_action(5500, (0.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "S2")
        );

        assert!(matches!(
            dispatch
                .get_action(5500, (-1.0, 1.0, 0.0), 1.0, Some(1.0))
                .await
                .unwrap(),
            Action::Fill
        ));

        assert!(
            matches!(dispatch .get_action(5500, (-1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A6")
        );
        assert!(
            matches!(dispatch .get_action(5500, (0.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "P2")
        );
        assert!(
            dispatch
                .get_action(5500, (0.0, 0.0, 0.0), 1.0, Some(1.0))
                .await
                .is_none()
        );
    }
}
