use crate::{
    client::{Client, SessionMap},
    http::{HttpStream, request::HttpRequest, response::HttpResponse},
    router::Router,
};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::{net::TcpListener, sync::Mutex};

pub struct Server {
    listener: TcpListener,
    router: Arc<Router>,
    global: GlobalMap,
}

#[forbid(clippy::unwrap_used)]
impl Server {
    pub async fn new<A: tokio::net::ToSocketAddrs>(
        addr: A,
        router: Router,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Server {
            listener,
            router: Arc::new(router),
            global: GlobalMap::default(),
        })
    }

    pub async fn run(mut self) -> std::io::Result<()> {
        loop {
            let (raw_stream, _) = match self.listener.accept().await {
                Ok(p) => p,
                Err(_e) => continue,
            };
            let mut http_stream = HttpStream::from_tcpstream(raw_stream);
            println!("Spawning");
            tokio::spawn(create_and_run_client(
                http_stream,
                self.router.clone(),
                self.global.clone(),
            ));
        }
    }
}

async fn create_and_run_client(stream: HttpStream, router: Arc<Router>, global: GlobalMap) {
    Client {
        stream,
        router,
        global,
        session: SessionMap::default(),
    }
    .run()
    .await;
}

#[derive(Default, Clone)]
pub struct GlobalMap {
    pub(crate) map: Arc<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}
