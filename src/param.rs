pub trait Param: 'static + Send + Sync {
    type Item;
    fn fetch() -> Self::Item;
}

macro_rules! param_tuple {
    { $($param:ident)* } => {
        impl<$($param,)*> Param for ($($param,)*)
        where $($param: Param,)*
        {
            type Item = ($($param::Item,)*);
            #[allow(clippy::unused_unit)]
            fn fetch() -> Self::Item {
                ($($param::fetch(),)*)
            }
        }
    };
}

param_tuple! {}
param_tuple! { A }
param_tuple! { A B }
param_tuple! { A B C }
param_tuple! { A B C D }

impl Param for String {
    type Item = String;
    fn fetch() -> Self::Item {
        String::from("Fetched")
    }
}

impl Param for usize {
    type Item = usize;
    fn fetch() -> Self::Item {
        12
    }
}
