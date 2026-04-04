use std::{marker::PhantomData, pin::Pin};

use crate::{
    client::SessionMap,
    http::{request::HttpRequest, response::HttpResponse},
    param::Param,
    router::Response,
    server::GlobalMap,
};

pub trait Handler: 'static + Send + Sync {
    fn run(
        &self,
        session: &mut SessionMap,
        global: &mut GlobalMap,
        request: &HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + Sync>>;
}

pub struct HandlerFunc<F, P> {
    pub(crate) f: F,
    pub(crate) _p: PhantomData<P>,
}

impl<F, P, Fut> Handler for HandlerFunc<F, P>
where
    Fut: Future<Output = Response> + 'static + Send + Sync,
    P: Param + 'static + Send + Sync,
    F: for<'a> Fn(P::Item<'a>) -> Fut + 'static + Send + Sync,
{
    fn run(
        &self,
        session: &mut SessionMap,
        global: &mut GlobalMap,
        request: &HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + Sync>> {
        Box::pin((self.f)(P::fetch(session, global, request)))
    }
}
