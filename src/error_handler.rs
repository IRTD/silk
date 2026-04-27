use std::fmt::Debug;

use crate::{
    handler::{HandlerFunc, Service},
    router::Response,
};

pub struct ErrorHandler {
    pub(crate) not_found: Box<dyn Service>,
}

impl Debug for ErrorHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorHandler").finish()
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        ErrorHandler {
            not_found: Box::new(HandlerFunc::<_, ()>::new(_default_not_found)),
        }
    }
}

async fn _default_not_found() -> Response {
    Response::html("Page not Found").with_status(crate::http::response::StatusCode::NotFound)
}
