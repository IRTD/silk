use tracing::{debug, instrument};

use crate::{
    handler::{HandlerResources, Service},
    http::{
        HttpStream, HttpStreamError, Method,
        path::{PathVariables, SegmentParseError, ServiceCollection},
        response::StatusCode,
    },
    middleware::Middleware,
    router::{Response, Router},
    server::GlobalMap,
};
use std::sync::Arc;

pub struct Client {
    pub(crate) router: Arc<Router>,
    pub(crate) stream: HttpStream,
    pub(crate) global: Arc<GlobalMap>,
    pub(crate) middleware: Arc<Vec<Middleware>>,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("{0}")]
    HttpStream(#[from] HttpStreamError),
    #[error("{0}")]
    PathError(#[from] SegmentParseError),
}

impl Client {
    pub async fn handle(mut self) -> Result<(), ClientError> {
        'main: loop {
            let http_req = self.stream.next_request().await?;
            debug!(request = ?http_req);

            let (service, path_vars) = match self.router.get_route(&http_req.header.path) {
                Some(Ok((service, path_vars))) => (
                    service
                        .method(&http_req.header.method)
                        .unwrap_or(self.router.not_found()),
                    path_vars,
                ),
                Some(Err(e)) => return Err(ClientError::PathError(e)),
                None => (self.router.not_found(), PathVariables::new()),
            };

            let mut resources = HandlerResources::new(http_req, &self.global, path_vars);

            for middle in self.middleware.iter() {
                let response = middle.run.run(&mut resources).await;
                if StatusCode::Ok != response.status {
                    let http_resp = response.into_http_response(resources.request);
                    self.stream.send_response(http_resp).await?;
                    continue 'main;
                }
            }

            let response = service.run(&mut resources).await;
            let http_resp = response.into_http_response(resources.request);

            debug!(response = ?http_resp);
            self.stream.send_response(http_resp).await?;
        }
    }
}
