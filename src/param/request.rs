use std::{collections::HashMap, ops::Deref};

use crate::{http::request::HttpRequest, param::Param};

pub struct Request<T>(T);
impl<T> Param for Request<T>
where
    T: RequestExtractor + Send + Sync + 'static,
{
    fn fetch(resources: &crate::handler::HandlerResources<'_>) -> Self {
        Request(T::from_request(resources.request))
    }
}

impl<T> Deref for Request<T>
where
    T: Deref,
{
    type Target = T::Target;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

pub trait RequestExtractor {
    fn from_request(req: &HttpRequest) -> Self;
}

pub struct Headers(HashMap<String, String>);
impl RequestExtractor for Headers {
    fn from_request(req: &HttpRequest) -> Self {
        Headers(req.header.headers.clone())
    }
}

impl Deref for Headers {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Body(String);
impl RequestExtractor for Body {
    fn from_request(req: &HttpRequest) -> Self {
        Body(req.content.clone())
    }
}

impl Deref for Body {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
