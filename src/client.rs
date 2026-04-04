use crate::{
    http::{
        HttpStream, HttpStreamError, Method,
        request::HttpRequest,
        response::{ContentType, HttpResponse, HttpResponseHeader},
    },
    router::{Route, Router},
    server::GlobalMap,
};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub struct Client {
    pub(crate) stream: HttpStream,
    pub(crate) router: Arc<Router>,
    pub(crate) global: GlobalMap,
    pub(crate) session: SessionMap,
}

#[forbid(clippy::unwrap_used)]
impl Client {
    pub async fn run(mut self) {
        loop {
            let req = match self.stream.next_request().await {
                Ok(req) => req,
                Err(_e) => {
                    dbg!(_e);
                    return;
                }
            };
            // TODO
            dbg!(&req);
            let route = match req.header.method {
                Method::Get => Route::Get(req.header.path.clone()),
                _ => todo!(),
            };
            let f = match self.router.get_route(route) {
                Some(f) => f,
                None => match self.router.fallback() {
                    Some(fallback) => fallback,
                    None => {
                        if self.not_found(req.header.protocol.clone()).await.is_err() {
                            return;
                        }
                        continue;
                    }
                },
            };

            let resp = f.run(&mut self.session, &mut self.global, &req).await;
            let http_resp = HttpResponse {
                header: HttpResponseHeader {
                    protocol: req.header.protocol,
                    status_code: resp.status,
                    reason_phrase: String::new(),
                },
                content_type: resp.content_type,
                content_length: resp.body.len(),
                body: resp.body,
            };

            if self.stream.send_response(http_resp).await.is_err() {
                return;
            }
        }
    }

    async fn not_found(&mut self, protocol: String) -> Result<(), HttpStreamError> {
        let err_resp = HttpResponse {
            header: HttpResponseHeader {
                status_code: crate::http::response::StatusCode::NotFound,
                protocol,
                reason_phrase: String::new(),
            },
            content_type: crate::http::response::ContentType::TextPlain,
            content_length: 0,
            body: String::new(),
        };
        self.stream.send_response(err_resp).await
    }
}

#[derive(Default)]
pub struct SessionMap {
    pub(crate) map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}
