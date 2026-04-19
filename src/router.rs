use std::{collections::HashMap, marker::PhantomData};

use crate::{
    handler::{Handler, HandlerFunc, Service},
    http::{
        Method,
        path::{HttpNodeTree, SegmentParseError, ServiceCollection},
    },
    param::Param,
};

#[derive(Default)]
pub struct Router {
    routes: HttpNodeTree,
}

impl Router {
    pub fn route(
        &mut self,
        path: impl ToString,
        services: ServiceCollection,
    ) -> Result<(), SegmentParseError> {
        self.routes.add_service(path, services)
    }
}

pub struct Response {}
