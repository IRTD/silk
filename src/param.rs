use crate::{handler::HandlerResources, http::path::PathVariables};

pub trait Param: 'static + Send + Sync {
    type Item;
    fn fetch(resources: &HandlerResources<'_>) -> Self::Item;
}

macro_rules! param_tuple {
    { $($param:ident)* } => {
        impl<$($param,)*> Param for ($($param,)*)
        where $($param: Param,)*
        {
            type Item = ($($param::Item,)*);
            #[allow(clippy::unused_unit)]
            fn fetch(resources: &HandlerResources<'_>) -> Self::Item {
                ($($param::fetch(resources),)*)
            }
        }
    };
}

param_tuple! {}
param_tuple! { A }
param_tuple! { A B }
param_tuple! { A B C }
param_tuple! { A B C D }
