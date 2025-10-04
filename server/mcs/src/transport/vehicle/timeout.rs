use std::sync::Arc;

use tokio::{sync::RwLock, time};

use chrono::Local;

use crate::{constant, transport::vehicle::State};

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
    use dotenvy::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use tokio::time;

    use crate::transport::{
        db_manager::DbManager,
        track::Graph,
        vehicle::{State, Vehicle},
    };

    use super::*;

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
    async fn vehicle_timeout() {
        let track_graph = Arc::new(get_track_graph().await);
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
