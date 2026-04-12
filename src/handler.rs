use std::{marker::PhantomData, pin::Pin};

use crate::{param::Param, router::Response};

pub trait Service {
    fn run(&self) -> BoxedHandlerOutput;
}

impl<F, P> Service for HandlerFunc<F, P>
where
    F: Handler<P::Item>,
    P: Param,
{
    fn run(&self) -> BoxedHandlerOutput {
        self.f.call(P::fetch())
    }
}

pub struct HandlerFunc<F, P> {
    pub(crate) f: F,
    pub(crate) _p: PhantomData<P>,
}

pub type BoxedHandlerOutput = Pin<Box<dyn Future<Output = Response>>>;

pub trait Handler<Args> {
    fn call(&self, args: Args) -> BoxedHandlerOutput;
}

macro_rules! handler_tuple {
    { $($param:ident)* } => {
        impl<F, Fut, $($param,)*> Handler<($($param,)*)> for F
        where Fut: Future<Output = Response> + 'static,
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
