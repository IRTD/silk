use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use tokio::{net::TcpListener, sync::RwLock};
use tracing::{Level, event, instrument};

use crate::{client::Client, http::HttpStream, router::Router};

#[derive(Default, Debug)]
pub struct GlobalMap {
    map: HashMap<TypeId, Arc<Box<dyn Any + Send + Sync>>>,
}

impl Deref for GlobalMap {
    type Target = HashMap<TypeId, Arc<Box<dyn Any + Send + Sync>>>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl GlobalMap {
    pub fn add_resource<T: Any + Send + Sync>(mut self, resource: T) -> Self {
        self.map
            .insert(TypeId::of::<T>(), Arc::new(Box::new(resource)));
        self
    }
}

#[derive(Debug)]
pub struct Server {
    router: Arc<Router>,
    global: Arc<GlobalMap>,
}

impl Server {
    pub fn new(router: Router) -> Self {
        Server {
            router: Arc::new(router),
            global: Arc::default(),
        }
    }

    pub fn with_global(mut self, global: GlobalMap) -> Self {
        self.global = Arc::new(global);
        self
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
                global: self.global.clone(),
            };

            tokio::spawn(client.handle());
        }
        Ok(())
    }
}
