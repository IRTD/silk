use silk::{
    http::path::ServiceCollection,
    param::{
        global::Global,
        path::{Path, PathExtractor},
        request::{Headers, Request},
    },
    router::{Response, Router},
    server::{GlobalMap, Server},
};
use tokio::net::TcpListener;
use tracing_subscriber::prelude::*;

pub struct User(String);
impl PathExtractor for User {
    fn name() -> &'static str {
        "user"
    }

    fn parse(input: Option<&String>) -> Self {
        User(input.unwrap().to_owned())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let router = Router::default()
        .route(
            "/hello/{user}",
            ServiceCollection::default().set_get::<_, (Path<User>, Global<String>)>(hello_page),
        )
        .route(
            "/headers",
            ServiceCollection::default().set_get::<_, (Request<Headers>,)>(header_show),
        );

    let global = GlobalMap::default().add_resource(String::from("Welcome"));
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    Server::new(router)
        .with_global(global)
        .run(listener)
        .await
        .unwrap();
}

async fn hello_page(user: Path<User>, welcome_msg: Global<String>) -> Response {
    Response::html(format!("{}, {}!", *welcome_msg, user.0))
}

async fn header_show(headers: Request<Headers>) -> Response {
    Response::html(format!("{:#?}", *headers))
}
