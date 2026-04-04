use crate::{http::request::HttpRequest, param::Param, server::GlobalMap};
use std::any::TypeId;

pub struct Global<T>(T);
impl<T: 'static> Param for Global<T> {
    type Item<'a> = Global<&'a T>;
    fn fetch<'a>(
        session: &'a mut crate::client::SessionMap,
        global: &'a mut GlobalMap,
        req: &'a HttpRequest,
    ) -> Self::Item<'a> {
        Global(
            global
                .map
                .get(&TypeId::of::<T>())
                .unwrap()
                .downcast_ref()
                .unwrap(),
        )
    }
}
