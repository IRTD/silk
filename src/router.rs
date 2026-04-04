use crate::{
    client::SessionMap,
    handler::{Handler, HandlerFunc},
    http::{
        request::HttpRequest,
        response::{ContentType, HttpResponse, StatusCode},
    },
    param::{Param, session::Session},
    server::GlobalMap,
};

use std::{collections::HashMap, marker::PhantomData, pin::Pin, sync::Arc};

pub struct Router {
    routes: HashMap<Route, Box<dyn Handler>>,
    fallback: Option<Box<dyn Handler>>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
            fallback: None,
        }
    }

    pub fn set_fallback<H>(mut self, fallback: H) -> Self
    where
        H: Handler + 'static,
    {
        self.fallback = Some(Box::new(fallback));
        self
    }

    pub fn fallback(&self) -> Option<&Box<dyn Handler>> {
        self.fallback.as_ref()
    }

    pub fn add_route<F, P, Fut>(mut self, route: Route, handler: F) -> Self
    where
        Fut: Future<Output = Response> + 'static + Send + Sync,
        P: Param + 'static + Send + Sync,
        F: Fn(P::Item<'_>) -> Fut + 'static + Send + Sync,
    {
        self.routes.insert(
            route,
            Box::new(HandlerFunc {
                f: handler,
                _p: PhantomData,
            }),
        );
        self
    }

    pub fn get_route(self: &Arc<Self>, route: Route) -> Option<&Box<dyn Handler>> {
        self.routes.get(&route)
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum Route {
    Get(String),
    Post(String),
    Put(String),
}

pub struct Response {
    pub content_type: ContentType,
    pub body: String,
    pub status: StatusCode,
}

impl Response {
    pub fn html(body: String) -> Self {
        Response {
            content_type: ContentType::TextHtml,
            body,
            status: StatusCode::Ok,
        }
    }
}
