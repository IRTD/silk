use crate::{client::SessionMap, http::request::HttpRequest, server::GlobalMap};

pub mod global;
pub mod path;
pub mod session;

pub trait Param {
    type Item<'a>;
    fn fetch<'a>(
        session: &'a mut SessionMap,
        global: &'a mut GlobalMap,
        req: &'a HttpRequest,
    ) -> Self::Item<'a>;
}
