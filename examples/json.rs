use serde::Deserialize;
use silk::{
    param::json::Json,
    post,
    router::{Response, Router},
    server::Server,
};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Name {
    name: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let router = Router::default().route("/hello", post(json_hello));
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    Server::new(router).run(listener).await.unwrap();
}

async fn json_hello(name: Json<Name>) -> Response {
    dbg!(&*name);
    Response::ok()
}
