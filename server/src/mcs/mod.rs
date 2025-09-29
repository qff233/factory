mod db;
pub mod prelude;
mod transport;

use std::sync::Arc;

use prelude::*;
use tokio::sync::mpsc;

use crate::mcs::{
    db::DbClient,
    transport::{DispatchExec, PingPong, TrackGraph},
};

pub struct McsConfig {
    pub sql: Arc<sqlx::PgPool>,
    pub transport_server_addr: String,
    pub timeout: i64,
    pub tool_warn_level: f32,
    pub track_graph: TrackGraph,
}

pub struct Mcs {
    transport_server_addr: String,
    vehicle_pingpong: PingPong,
    transport_server_builder: transport::ServerBuilder,
    db_client: DbClient,
}

impl Mcs {
    pub fn new(config: McsConfig) -> Self {
        let (sender, receiver) = mpsc::channel(200);
        let vehicle_pingpong = PingPong::new(sender.clone(), config.timeout);
        let dispatch_exec = DispatchExec::new(receiver, config.tool_warn_level, config.track_graph);
        let transport_server_addr = config.transport_server_addr;
        let mut transport_server_builder = transport::ServerBuilder::new(dispatch_exec);
        let db_client = DbClient::new(config.sql);

        let register = transport_server_builder.register();
        register.register_async_method("trans_item", async |params, dispatch_exec, _| {});

        Self {
            vehicle_pingpong,
            transport_server_addr,
            transport_server_builder,
            db_client,
        }
    }
    // pub fn new(config: MCSConfig) -> Self {
    //     let (vehicle_pingpong, vehicle_request, vehicle_exec) =
    //         vehicle::dispatch(config.timeout, config.tool_warn_level, config.track_graph);
    //     Self {
    //         sql: config.sql,
    //         vehicle_pingpong,
    //         vehicle_request,
    //         vehicle_exec,
    //     }
    // }

    pub async fn run(self) {
        let transport_server = self.transport_server_builder.build(self.transport_server_addr).await.expect("Start transport server error!");
        tokio::spawn(async move {
            self.vehicle_pingpong.task().await;
        })
        .await
        .unwrap();
        tokio::spawn(async move {
            transport_server.task().await;
        })
        .await
        .unwrap();
    }
}
