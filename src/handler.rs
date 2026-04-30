use std::{marker::PhantomData, pin::Pin};

use crate::{
    http::{path::PathVariables, request::HttpRequest},
    param::Param,
    router::Response,
    server::GlobalMap,
};

#[derive(Debug)]
pub struct HandlerResources<'a> {
    pub path_vars: PathVariables,
    pub request: HttpRequest,
    pub global: &'a GlobalMap,
}

impl<'a> HandlerResources<'a> {
    pub fn new(request: HttpRequest, global: &'a GlobalMap, path_vars: PathVariables) -> Self {
        HandlerResources {
            path_vars,
            request,
            global,
        }
    }
}

pub trait Service: Send + Sync {
    fn run(&self, resources: &mut HandlerResources<'_>) -> BoxedHandlerOutput;
}

impl<F, P> Service for HandlerFunc<F, P>
where
    F: Handler<P> + Send + Sync,
    P: Param,
{
    fn run(&self, resources: &mut HandlerResources<'_>) -> BoxedHandlerOutput {
        self.f.call(P::fetch(resources))
    }
}

pub struct HandlerFunc<F, P> {
    pub(crate) f: F,
    pub(crate) _p: PhantomData<P>,
}

impl<F, P> HandlerFunc<F, P>
where
    F: Handler<P> + Send + Sync,
    P: Param,
{
    pub fn new(f: F) -> Self {
        HandlerFunc {
            f,
            _p: PhantomData::<P>,
        }
    }
}

pub type BoxedHandlerOutput = Pin<Box<dyn Future<Output = Response> + Send + Sync>>;

pub trait Handler<Args> {
    fn call(&self, args: Args) -> BoxedHandlerOutput;
}

macro_rules! handler_tuple {
    { $($param:ident)* } => {
        impl<F, Fut, $($param,)*> Handler<($($param,)*)> for F
        where Fut: Future<Output = Response> + 'static + Send + Sync,
              F: Fn($($param,)*) -> Fut,
        {
            #[allow(non_snake_case)]
            fn call(&self, ($($param,)*): ($($param,)*)) -> BoxedHandlerOutput {
                Box::pin((self)($($param,)*))
            }
        }
    };
}

handler_tuple! {}
handler_tuple! { A }
handler_tuple! { A B }
handler_tuple! { A B C }
handler_tuple! { A B C D }
