#[derive(Default, Debug)]
pub struct HttpResponse {
    pub header: HttpResponseHeader,
    pub content_type: ContentType,
    pub content_length: usize,
    pub body: String,
}

impl HttpResponse {
    pub fn new(header: HttpResponseHeader, content_type: ContentType, body: impl ToString) -> Self {
        let body = body.to_string();
        HttpResponse {
            header,
            content_type,
            content_length: body.len(),
            body,
        }
    }
}

impl ToString for HttpResponse {
    fn to_string(&self) -> String {
        [
            self.header.to_string(),
            format!("Content-Type: {}", self.content_type),
            format!("Content-Length: {}\r\n", self.content_length),
            self.body.clone(),
        ]
        .join("\r\n")
    }
}

#[derive(Default, Debug)]
pub struct HttpResponseHeader {
    pub protocol: String,
    pub status_code: StatusCode,
    pub reason_phrase: String,
}

impl ToString for HttpResponseHeader {
    fn to_string(&self) -> String {
        format!(
            "{} {} {} ",
            self.protocol, self.status_code as u16, self.reason_phrase
        )
    }
}

#[derive(Default, Debug, strum::Display)]
pub enum ContentType {
    #[strum(serialize = "text/html")]
    TextHtml,
    #[strum(serialize = "application/x-www-form-urlencoded")]
    AppForm,
    #[strum(serialize = "multipart/form-data")]
    FormData,
    #[default]
    #[strum(serialize = "text/plain")]
    TextPlain,
}

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u16)]
pub enum StatusCode {
    #[default]
    Ok = 200,
    Created = 201,
    Accepted = 202,

    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    Teapot = 418,

    Internale = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    HttpVersionUnsupported = 505,
}
