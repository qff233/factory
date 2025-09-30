use tokio::net::ToSocketAddrs;

pub struct Server {
    server: jsonrpsee::server::Server,
    module: jsonrpsee::RpcModule<ScheduleExec>,
}

impl Server {
    pub async fn task(self) {
        let handle = self.server.start(self.module);
        handle.stopped().await
    }
}

pub struct ServerBuilder {
    module: jsonrpsee::RpcModule<ScheduleExec>,
}

impl ServerBuilder {
    pub fn new(dispatch_exec: ScheduleExec) -> Self {
        let module = jsonrpsee::RpcModule::new(dispatch_exec);
        Self { module }
    }

    pub fn register(&mut self) -> &mut jsonrpsee::RpcModule<ScheduleExec> {
        &mut self.module
    }

    pub async fn build(self, addr: impl ToSocketAddrs) -> std::io::Result<Server> {
        let server = jsonrpsee::server::ServerBuilder::new().build(addr).await?;
        Ok(Server {
            server,
            module: self.module,
        })
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use crate::transport::{TrackGraph, track::TrackGraphBuilder};

    use super::*;
    #[tokio::test]
    async fn jsonrpc_server() {
        let track_graph = TrackGraphBuilder::new().build();
        let (sender, receiver) = mpsc::channel(20);
        let dispatch_exec = ScheduleExec::new(receiver, 0.1, track_graph);
        let server = ServerBuilder::new(dispatch_exec.await).build("0.0.0.0:5000");
    }
}
