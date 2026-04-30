use crate::handler::Service;

pub struct Middleware {
    pub(crate) setup: Box<dyn Service>,
    pub(crate) run: Box<dyn Service>,
}

impl Middleware {
    pub fn new<S1, S2>(setup: S1, run: S2) -> Self
    where
        S1: Service + 'static,
        S2: Service + 'static,
    {
        Middleware {
            setup: Box::new(setup),
            run: Box::new(run),
        }
    }
}
