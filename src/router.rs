use crate::{
    error_handler::ErrorHandler,
    handler::Service,
    http::{
        path::{HttpNodeTree, PathVariables, SegmentParseError, ServiceCollection},
        request::HttpRequest,
        response::{ContentType, HttpResponse, HttpResponseHeader, StatusCode},
    },
};

#[derive(Default, Debug)]
pub struct Router {
    routes: HttpNodeTree,
    error_handler: ErrorHandler,
}

impl Router {
    pub fn with_error_handler(error_handler: ErrorHandler) -> Self {
        Router {
            error_handler,
            ..Default::default()
        }
    }

    pub fn route(mut self, path: impl ToString, services: ServiceCollection) -> Self {
        self.routes.add_service(path, services).unwrap();
        self
    }

    pub fn not_found(&self) -> &Box<dyn Service> {
        &self.error_handler.not_found
    }

    pub fn get_route(
        &self,
        path: impl ToString,
    ) -> Option<Result<(&ServiceCollection, PathVariables), SegmentParseError>> {
        self.routes
            .get_node(path)
            .map(|r| r.map(|(node, vars)| (&node.methods, vars)))
    }
}

#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    content_type: ContentType,
    reason_phrase: String,
    body: String,
}

impl Response {
    pub fn html(body: impl ToString) -> Self {
        Response {
            status: StatusCode::Ok,
            content_type: ContentType::TextHtml,
            reason_phrase: String::new(),
            body: body.to_string(),
        }
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn with_reasoning(mut self, reason: impl ToString) -> Self {
        self.reason_phrase = reason.to_string();
        self
    }

    pub fn into_http_response(self, origin: HttpRequest) -> HttpResponse {
        HttpResponse {
            header: HttpResponseHeader {
                protocol: origin.header.protocol,
                status_code: self.status,
                reason_phrase: self.reason_phrase,
            },
            content_type: self.content_type,
            content_length: self.body.len(),
            body: self.body,
        }
    }
}
