use std::{any::TypeId, ops::Deref};

use super::Param;
use crate::{client::SessionMap, http::request::HttpRequest, server::GlobalMap};

pub struct Session<T>(T);
impl<T: 'static> Param for Session<T> {
    type Item<'a> = Session<T>;
    fn fetch<'a>(
        session: &'a mut SessionMap,
        global: &'a mut GlobalMap,
        req: &'a HttpRequest,
    ) -> Self::Item<'a> {
        Session(
            *session
                .map
                .remove(&TypeId::of::<T>())
                .unwrap()
                .downcast()
                .unwrap(),
        )
    }
}

impl<T> Deref for Session<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
