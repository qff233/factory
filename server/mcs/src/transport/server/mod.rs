use crate::transport::{prelude::Position, schedule::ScheduleExec};
use serde::{Deserialize, Serialize};
use tokio::net::ToSocketAddrs;

pub struct Server {
    server: jsonrpsee::server::Server,
    module: jsonrpsee::RpcModule<ScheduleExec>,
}

impl Server {
    pub async fn run(addr: impl ToSocketAddrs, schedule_exec: ScheduleExec) {
        let mut module = jsonrpsee::RpcModule::new(schedule_exec);
        Self::register_method(&mut module);
        let server = jsonrpsee::server::ServerBuilder::new()
            .build(addr)
            .await
            .unwrap();
        let handle = server.start(module);
        tokio::spawn(async move {
            handle.stopped().await;
        });
    }

    fn register_method(module: &mut jsonrpsee::RpcModule<ScheduleExec>) {
        module
            .register_async_method("vehicle_get_action", async |params, schedule_exec, _| {
                #[derive(Deserialize, Debug)]
                struct Params {
                    id: i32,
                    position: (f64, f64, f64),
                    battery_level: f32,
                    tool_level: Option<f32>,
                }
                #[derive(Serialize, Debug)]
                struct Response {
                    action: String,
                    position: Position,
                }
                match params.parse::<Params>() {
                    Ok(params) => {
                        let position: Position = params.position.into();
                        let action = schedule_exec
                            .get_action(
                                params.id,
                                position,
                                params.battery_level,
                                params.tool_level,
                            )
                            .await;
                        format!("{:?}", action)
                    }
                    Err(e) => {
                        println!("parse error {:#?}", e);
                        "".to_string()
                    }
                }
            })
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    use crate::{db_manager::DbManager, transport::track::Graph};
    use dotenvy::dotenv;
    use jsonrpsee::core::client::ClientT;
    use sqlx::postgres::PgPoolOptions;
    use tokio::time::sleep;

    #[tokio::test]
    async fn jsonrpc_server() {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        println!("Connecting to database: {}", database_url);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool");
        let db = DbManager::new(pool);
        let track_graph = Graph::new(db.clone()).await;
        let schedule_exec = ScheduleExec::new(track_graph, db).await;
        Server::run("0.0.0.0:5000", schedule_exec).await;

        // sleep(Duration::from_secs(9999999999)).await;
        // thread::sleep(Duration::from_secs(99999999999999));
        // let url = "http://127.0.0.1:5000".to_string();
        // let client = jsonrpsee::http_client::HttpClient::builder()
        //     .build(url)
        //     .unwrap();

        // let json = serde_json::to_string(&params).unwrap();
        // println!("request json: {}", json);
        // let params: serde_json::Map<String, serde_json::Value> =
        //     serde_json::from_str(&json).unwrap();
        // let response: Result<String, _> = client.request("vehicle_get_action", params).await;
        // println!("response: {:?}", response);
    }
}
