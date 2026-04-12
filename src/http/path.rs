use crate::{handler::Service, http::Method};

pub struct HttpNodeTree {
    root: HttpNode,
}

pub struct HttpNode {
    segment: Segment,
    leaves: Vec<HttpNode>,
    methods: ServiceCollection,
}

pub struct ServiceCollection {
    get: Option<Box<dyn Service>>,
    post: Option<Box<dyn Service>>,
}

pub enum Segment {
    Static(String),
    Pattern { reference: String },
}

pub struct RequestPath {
    segments: Vec<Segment>,
}
