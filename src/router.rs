use std::{collections::HashMap, marker::PhantomData};

use crate::{
    handler::{Handler, HandlerFunc, Service},
    http::Method,
    param::Param,
};

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Route {
    method: Method,
    path: &'static str,
}

impl Route {
    pub fn get(path: &'static str) -> Self {
        Route {
            method: Method::Get,
            path,
        }
    }

    pub fn post(path: &'static str) -> Self {
        Route {
            method: Method::Post,
            path,
        }
    }
}

#[derive(Default)]
pub struct Router {
    routes: HashMap<Route, Box<dyn Service>>,
}

impl Router {
    pub fn route<H, P>(mut self, route: Route, handler: H) -> Self
    where
        H: Handler<P::Item> + 'static,
        P: Param,
    {
        self.routes.insert(
            route,
            Box::new(HandlerFunc {
                f: handler,
                _p: PhantomData::<P>,
            }),
        );
        self
    }

    pub(crate) fn get_route(&self, route: &Route) -> Option<&dyn Service> {
        self.routes.get(route).map(|s| &**s)
    }
}

pub struct Response {}
