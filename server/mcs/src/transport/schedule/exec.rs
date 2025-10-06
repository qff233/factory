use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, mpsc};
use tracing::warn;

use crate::{
    constant,
    db_manager::DbManager,
    transport::{
        prelude::Position,
        schedule::{action_planner::ActionPlanner, state_update::StateUpdate},
        track::Graph,
        vehicle::{self, Action, Vehicle},
    },
};

#[derive(Debug)]
pub struct ScheduleExec {
    track_graph: Arc<Graph>,
    vehicles: Arc<RwLock<HashMap<i32, Vehicle>>>,
    vehicle_event_sender: mpsc::Sender<vehicle::Event>,
}

impl ScheduleExec {
    pub async fn new(track_graph: Graph, db: Arc<DbManager>) -> Self {
        let vehicles = Arc::new(RwLock::new(HashMap::new()));
        let track_graph = Arc::new(track_graph);
        let (vehicle_event_sender, vehicle_event_receiver) = mpsc::channel(50);

        ActionPlanner::run(vehicles.clone(), track_graph.clone(), db.clone());
        StateUpdate::run(vehicle_event_receiver, db);
        Self {
            track_graph,
            vehicles,
            vehicle_event_sender,
        }
    }

    pub async fn get_action(
        &self,
        id: i32,
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
                vehicle.set_event_sender(self.vehicle_event_sender.clone());
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
    use crate::transport::schedule::adder::ScheduleAdder;
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
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        println!("Connecting to database: {}", database_url);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool");
        let db = DbManager::new(pool);
        let mut dispatch = ScheduleExec::new(track_graph, db.clone()).await;

        let mut adder = ScheduleAdder::new(db);
        adder.trans_items("S2", "S1").await.unwrap();
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
        println!("{:#?}", dispatch.vehicles.read().await.get(&2500).unwrap());
        assert!(
            matches!(dispatch.get_action(2500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );
        assert!(
            matches!(dispatch.get_action(2500, (0.0, 2.0, 0.0), 1.0, Some(1.0))
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

        adder.trans_fluid("S1", "S2").await.unwrap();
        // Yield to recv next task
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        assert!(
            matches!(dispatch.get_action(5500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );
        println!("{:#?}", dispatch.vehicles.read().await.get(&5500).unwrap());
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
            Action::Suck
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
            .await.unwrap(), Action::Move(node) if node.name == "A2")
        );
        assert!(
            matches!(dispatch .get_action(5500, (1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A1")
        );
        assert!(
            matches!(dispatch .get_action(5500, (2.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A4")
        );
        assert!(
            matches!(dispatch .get_action(5500, (2.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A3")
        );
        assert!(
            matches!(dispatch .get_action(5500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );
        assert!(
            matches!(dispatch .get_action(5500, (0.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "S3")
        );
        assert!(matches!(
            dispatch
                .get_action(5500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
                .await
                .unwrap(),
            Action::Drop
        ));
        assert!(
            matches!(dispatch .get_action(5500, (-1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name == "A5")
        );
        assert!(
            matches!(dispatch .get_action(5500, (0.0, 2.0, 0.0), 1.0, Some(1.0))
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
