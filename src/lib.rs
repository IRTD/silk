#![allow(unused)]

pub mod client;
pub mod error_handler;
pub mod handler;
pub mod http;
pub mod param;
pub mod router;
pub mod server;

pub use tokio;

use crate::{handler::Handler, http::path::ServiceCollection};

pub fn get<F, P>(service: F) -> ServiceCollection
where
    F: Handler<P> + Send + Sync + 'static,
    P: param::Param,
{
    ServiceCollection::default().set_get(service)
}

pub fn post<F, P>(service: F) -> ServiceCollection
where
    F: Handler<P> + Send + Sync + 'static,
    P: param::Param,
{
    ServiceCollection::default().set_post(service)
}
