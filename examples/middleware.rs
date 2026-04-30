use std::{any::TypeId, ops::AddAssign};

use silk::{
    get,
    middleware::Middleware,
    param::{
        Param,
        global::Global,
        path::{Path, PathExtractor},
    },
    router::{Response, Router},
    server::Server,
};
use tokio::{net::TcpListener, sync::Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct Counter(usize);
impl Param for Counter {
    fn fetch(resources: &mut silk::handler::HandlerResources<'_>) -> Self {
        Counter(
            *resources
                .global
                .get(&TypeId::of::<Counter>())
                .unwrap()
                .downcast_ref()
                .unwrap(),
        )
    }
}

struct Name(String);
impl PathExtractor for Name {
    fn name() -> &'static str {
        "name"
    }

    fn parse(input: Option<&String>) -> Self {
        Name(input.unwrap().to_owned())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let router = Router::default()
        .route("/health", get(health))
        .route("/{name}", get(greeting));

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    Server::new(router)
        .add_resource(Mutex::new(Counter(0)))
        // .add_middleware(counter_middleware)
        .run(listener)
        .await
        .unwrap();
}

async fn greeting(path: Path<Name>) -> Response {
    Response::html(format!("Hello, {}", path.0))
}

async fn health(counter: Global<Mutex<Counter>>) -> Response {
    Response::html(format!(
        "Site has been loaded {} times",
        counter.lock().await.0
    ))
}
