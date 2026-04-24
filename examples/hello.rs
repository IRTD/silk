use silk::{
    handler::HandlerResources,
    http::path::ServiceCollection,
    param::Param,
    router::{Response, Router},
    server::Server,
};
use tokio::net::TcpListener;

struct User(String);

impl Param for User {
    fn fetch(resources: &HandlerResources<'_>) -> Self {
        User(
            resources
                .path_vars
                .unwrap()
                .get("user")
                .unwrap()
                .to_string(),
        )
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let router = Router::default()
        .route(
            "/hello/{user}",
            ServiceCollection::default().set_get::<_, (User, User)>(hello_page),
        )
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    Server::new(router).run(listener).await.unwrap();
}

async fn hello_page(user: User, _user1: User) -> Response {
    Response::html(format!("Hello, {}!", user.0))
}
