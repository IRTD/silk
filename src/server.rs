use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use tokio::net::TcpListener;
use tracing::{Level, event, instrument};

use crate::{
    client::Client,
    handler::{Handler, HandlerFunc, Service},
    http::HttpStream,
    param::Param,
    router::Router,
};

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
    pub fn add_resource<T: Any + Send + Sync>(&mut self, resource: T) {
        self.map
            .insert(TypeId::of::<T>(), Arc::new(Box::new(resource)));
    }
}

pub struct Server {
    router: Router,
    global: GlobalMap,
    middleware: Vec<Box<dyn Service>>,
}

impl Server {
    pub fn new(router: Router) -> Self {
        Server {
            router: router,
            global: GlobalMap::default(),
            middleware: Vec::new(),
        }
    }

    pub fn add_resource<T>(mut self, resource: T) -> Self
    where
        T: Any + 'static + Send + Sync,
    {
        self.global.add_resource(resource);
        self
    }

    pub fn add_middleware<F, P>(mut self, middleware: F) -> Self
    where
        F: Handler<P> + 'static + Send + Sync,
        P: Param,
    {
        self.middleware
            .push(Box::new(HandlerFunc::<_, P>::new(middleware)));
        self
    }

    pub async fn run(self, listener: TcpListener) -> tokio::io::Result<()> {
        let router = Arc::new(self.router);
        let global = Arc::new(self.global);
        let middleware = Arc::new(self.middleware);
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(p) => p,
                Err(_e) => continue,
            };
            event!(Level::DEBUG, "Accepted Connection");

            let client = Client {
                router: router.clone(),
                stream: HttpStream::from_tcpstream(stream),
                global: global.clone(),
                middleware: middleware.clone(),
            };

            tokio::spawn(client.handle());
        }
    }
}
