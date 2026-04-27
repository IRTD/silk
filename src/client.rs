use tracing::{Level, debug, event, instrument};

use crate::{
    handler::HandlerResources,
    http::{
        HttpStream, HttpStreamError, Method,
        path::{PathVariables, SegmentParseError, ServiceCollection},
    },
    router::{Response, Router},
    server::GlobalMap,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Client {
    pub(crate) router: Arc<Router>,
    pub(crate) stream: HttpStream,
    pub(crate) global: Arc<GlobalMap>,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("{0}")]
    HttpStream(#[from] HttpStreamError),
    #[error("{0}")]
    PathError(#[from] SegmentParseError),
}

impl Client {
    #[instrument(name = "Client::handle")]
    pub async fn handle(mut self) -> Result<(), ClientError> {
        loop {
            let http_req = self.stream.next_request().await?;
            debug!(request = ?http_req);
            let mut resources = HandlerResources::new(&http_req, &self.global);

            let response = match self.router.get_route(&http_req.header.path) {
                Some(res) => {
                    let (methods, path_vars) = res?;
                    resources.path_vars = Some(&path_vars);
                    self.run_service(methods, resources, &http_req.header.method)
                        .await
                }
                None => self.router.not_found().run(resources).await,
            };
            let http_resp = response.into_http_response(http_req);

            debug!(response = ?http_resp);
            self.stream.send_response(http_resp).await?;
        }
    }

    async fn run_service(
        &self,
        methods: &ServiceCollection,
        resources: HandlerResources<'_>,
        method: &Method,
    ) -> Response {
        match method {
            Method::Get => {
                methods
                    .get()
                    .unwrap_or(self.router.not_found())
                    .run(resources)
                    .await
            }
            Method::Post => {
                methods
                    .post()
                    .unwrap_or(self.router.not_found())
                    .run(resources)
                    .await
            }
            _ => todo!(),
        }
    }
}
