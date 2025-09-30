use std::sync::Arc;

use tokio::{sync::RwLock, time};

use chrono::Local;

use crate::{
    constant,
    transport::vehicle::{State, Vehicle},
};

#[derive(Debug)]
pub struct Timeout {
    time_stamp: Arc<RwLock<chrono::DateTime<Local>>>,
}

impl Timeout {
    pub fn new(state: Arc<RwLock<State>>) -> Self {
        let time_stamp = Arc::new(RwLock::new(Local::now()));

        let inner_time_stamp = time_stamp.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(time::Duration::from_secs(
                constant::VEHICLE_ONLINE_UPDATE_TIMEOUT as u64,
            ));
            loop {
                interval.tick().await;
                let now = chrono::Local::now();
                let dt = (now - *inner_time_stamp.read().await).num_seconds();
                // println!("detect! dt={}", dt);
                if dt > constant::VEHICLE_ONLINE_UPDATE_TIMEOUT {
                    *state.write().await = State::Offline;
                }
            }
        });

        Self { time_stamp }
    }

    pub async fn update(&mut self) {
        *self.time_stamp.write().await = chrono::Local::now();
    }
}

#[cfg(test)]
mod tests {
    use tokio::time;

    use crate::transport::{
        prelude::Side,
        track::{self, NodeType},
        vehicle::State,
    };

    use super::*;

    fn get_tarck_graph() -> track::TrackGraph {
        track::TrackGraphBuilder::new()
            .node("S1", (2.0, 3.0, 0.0), NodeType::ShippingDock(Side::PosZ))
            .node("P2", (0.0, 0.0, 0.0), NodeType::ParkingStation)
            .node("C1", (1.0, 0.0, 0.0), NodeType::ChargingStation)
            .node("P1", (2.0, 0.0, 0.0), NodeType::ParkingStation)
            .node("A1", (2.0, 1.0, 0.0), NodeType::Fork)
            .node("A2", (1.0, 1.0, 0.0), NodeType::Fork)
            .node("A3", (1.0, 2.0, 0.0), NodeType::Fork)
            .node("A4", (2.0, 2.0, 0.0), NodeType::Fork)
            .node("A5", (0.0, 2.0, 0.0), NodeType::Fork)
            .node("A6", (0.0, 1.0, 0.0), NodeType::Fork)
            .edge_double("P2", "A6")
            .edge_double("C1", "A2")
            .edge_double("P1", "A1")
            .edge_double("S1", "A4")
            .edge("A6", "A2")
            .edge("A2", "A1")
            .edge("A1", "A4")
            .edge("A4", "A3")
            .edge("A3", "A2")
            .edge("A3", "A5")
            .edge("A5", "A6")
            .build()
    }

    #[tokio::test]
    async fn overtime_vehicle() {
        let track_graph = Arc::new(get_tarck_graph());
        let vehicle = Arc::new(RwLock::new(Vehicle::new(2000, track_graph).await));

        assert!(matches!(
            *vehicle.read().await.state.read().await,
            State::Offline
        ));

        vehicle
            .write()
            .await
            .get_action(&(0.0, 0.0, 0.0).into(), 1.0)
            .await;
        assert!(matches!(
            *vehicle.read().await.state.read().await,
            State::Initing(_)
        ));

        time::sleep(time::Duration::from_secs(11)).await;
        assert!(matches!(
            *vehicle.read().await.state.read().await,
            State::Offline
        ));
    }
}
