pub mod global;
pub mod path;
pub mod request;

use crate::handler::HandlerResources;

pub trait Param: 'static + Send + Sync {
    fn fetch(resources: &HandlerResources<'_>) -> Self;
}

macro_rules! param_tuple {
    { $($param:ident)* } => {
        impl<$($param,)*> Param for ($($param,)*)
        where $($param: Param,)*
        {
            #[allow(clippy::unused_unit, unused_variables)]
            fn fetch(resources: &HandlerResources<'_>) -> Self{
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
