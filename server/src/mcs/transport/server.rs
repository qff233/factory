use tokio::net::ToSocketAddrs;

use crate::mcs::transport::DispatchExec;

pub struct Server {
    server: jsonrpsee::server::Server,
    module: jsonrpsee::RpcModule<DispatchExec>,
}

impl Server {
    pub async fn task(self) {
        let handle = self.server.start(self.module);
        handle.stopped().await
    }
}

pub struct ServerBuilder {
    module: jsonrpsee::RpcModule<DispatchExec>,
}

impl ServerBuilder {
    pub fn new(dispatch_exec: DispatchExec) -> Self {
        let module = jsonrpsee::RpcModule::new(dispatch_exec);
        Self { module }
    }

    pub fn register(&mut self) -> &mut jsonrpsee::RpcModule<DispatchExec>{
        &mut self.module
    }

    pub async fn build(self, addr: impl ToSocketAddrs) -> std::io::Result<Server> {
        let server = jsonrpsee::server::ServerBuilder::new().build(addr).await?;
        Ok(Server {
            server,
            module: self.module
        })
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn jsonrpc_server() {

    }
}
