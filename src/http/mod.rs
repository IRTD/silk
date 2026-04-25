use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use tracing::{debug, instrument};

use crate::http::{
    request::{HttpRequest, HttpRequestParser},
    response::HttpResponse,
};

pub mod path;
pub mod request;
pub mod response;

pub use request::Method;

#[derive(Debug)]
pub struct HttpStream {
    tcp: BufReader<TcpStream>,
}

impl HttpStream {
    pub fn from_tcpstream(tcp: TcpStream) -> Self {
        HttpStream {
            tcp: BufReader::new(tcp),
        }
    }

    pub async fn next_request(&mut self) -> Result<HttpRequest, HttpStreamError> {
        let mut buffer = [0; 4096];
        let read = self.tcp.read(&mut buffer).await?;

        let s = String::from_utf8_lossy(&buffer[..read]);
        Ok(HttpRequestParser::new(&s).parse()?)
    }

    pub async fn send_response(&mut self, response: HttpResponse) -> Result<(), HttpStreamError> {
        self.tcp.write_all(response.to_string().as_bytes()).await?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HttpStreamError {
    #[error("{0}")]
    ParseError(#[from] crate::http::request::ParseError),
    #[error("{0}")]
    IoError(#[from] tokio::io::Error),
}
