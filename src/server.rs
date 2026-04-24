use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use tokio::{net::TcpListener, sync::RwLock};
use tracing::{Level, event, instrument};

use crate::{client::Client, http::HttpStream, router::Router};

pub(crate) type GlobalMap = Arc<HashMap<TypeId, Arc<RwLock<Box<dyn Any>>>>>;

#[derive(Debug)]
pub struct Server {
    router: Arc<Router>,
    // global_map: GlobalMap,
}

impl Server {
    pub fn new(router: Router) -> Self {
        Server {
            router: Arc::new(router),
        }
    }

    #[instrument(name = "Server::run()")]
    pub async fn run(self, listener: TcpListener) -> tokio::io::Result<()> {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(p) => p,
                Err(_e) => continue,
            };
            event!(Level::DEBUG, "Accepted Connection");

            let client = Client {
                router: self.router.clone(),
                stream: HttpStream::from_tcpstream(stream),
            };

            tokio::spawn(client.handle());
        }
        Ok(())
    }
}
