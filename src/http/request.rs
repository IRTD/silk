use std::{collections::HashMap, str::FromStr};

#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    pub header: HttpRequestHeader,
    pub content: String,
}

impl Default for HttpRequest {
    fn default() -> Self {
        HttpRequest {
            header: HttpRequestHeader {
                method: Method::Get,
                path: String::new(),
                protocol: String::new(),
                misc: HashMap::new(),
            },
            content: String::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HttpRequestHeader {
    pub method: Method,
    pub path: String,
    pub protocol: String,
    pub misc: HashMap<String, String>,
}

#[derive(Debug, PartialEq, strum::EnumString)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Head,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

pub(crate) enum ParseStage {
    Method,
    Path,
    Protocol,
    Misc,
    Content,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid Header")]
    InvalidHeader,
}

pub struct HttpRequestParser {
    request: HttpRequest,
    buffer: String,
    cursor: usize,
    stage: ParseStage,
    input: Vec<char>,
}

impl HttpRequestParser {
    pub fn new(input: &str) -> Self {
        Self {
            request: HttpRequest::default(),
            buffer: String::new(),
            cursor: 0,
            stage: ParseStage::Method,
            input: input.chars().collect(),
        }
    }

    pub fn parse(mut self) -> Result<HttpRequest, ParseError> {
        loop {
            match self.stage {
                ParseStage::Method => {
                    if !self.collect_until(' ') {
                        return Err(ParseError::InvalidHeader);
                    }
                    self.request.header.method = Method::from_str(self.buffer.drain(..).as_str())
                        .map_err(|_| ParseError::InvalidHeader)?;
                    self.stage = ParseStage::Path;
                }
                ParseStage::Path => {
                    if !self.collect_until(' ') {
                        return Err(ParseError::InvalidHeader);
                    }
                    self.request.header.path = self.buffer.drain(..).collect();
                    self.stage = ParseStage::Protocol;
                }
                ParseStage::Protocol => {
                    if !self.collect_until('\r') {
                        return Err(ParseError::InvalidHeader);
                    }
                    self.request.header.protocol = self.buffer.drain(..).collect();
                    self.next().ok_or(ParseError::InvalidHeader)?;
                    self.stage = ParseStage::Misc;
                }
                ParseStage::Misc => {
                    if self.peek() == Some('\r') || self.peek() == Some('\n') {
                        self.stage = ParseStage::Content;
                        continue;
                    }
                    self.collect_until(':');
                    let key = self.buffer.drain(..).collect();
                    self.skip_while(|ch| ch == ' ');
                    self.collect_until('\r');
                    let value = self.buffer.drain(..).collect();
                    self.next();
                    self.request.header.misc.insert(key, value);
                }
                ParseStage::Content => {
                    self.skip_while(|ch| ['\r', '\n'].contains(&ch));
                    while let Some(ch) = self.next() {
                        self.request.content.push(ch);
                    }
                    break;
                }
            }
        }

        Ok(self.request)
    }

    pub fn next(&mut self) -> Option<char> {
        let ch = self.input.get(self.cursor)?;
        self.cursor += 1;
        Some(*ch)
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.cursor + 1).copied()
    }

    /// Pushes characters of the input into buffer until either the stop is met or EOF
    /// On stop: returns true
    /// On EOF: returns false
    ///
    /// *Note: Stop character will not be pushed onto the buffer*
    pub fn collect_until(&mut self, stop: char) -> bool {
        loop {
            let ch = match self.next() {
                Some(ch) => ch,
                None => return false,
            };

            if ch == stop {
                return true;
            }

            self.buffer.push(ch);
        }
    }

    pub fn skip_while<P>(&mut self, predicate: P)
    where
        P: Fn(char) -> bool,
    {
        loop {
            match self.next() {
                Some(ch) => {
                    if !predicate(ch) {
                        self.cursor -= 1;
                        return;
                    }
                }
                None => return,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_correct_http_request() {
        let req_str =
            "GET /home HTTP/1.1\r\nHost: example.com\r\nAccept-Language: en\r\n\r\nHello World";

        let mut misc = HashMap::new();
        misc.insert(String::from("Host"), String::from("example.com"));
        misc.insert(String::from("Accept-Language"), String::from("en"));

        let expected_req = HttpRequest {
            header: HttpRequestHeader {
                method: Method::Get,
                path: String::from("/home"),
                protocol: String::from("HTTP/1.1"),
                misc,
            },
            content: String::from("Hello World"),
        };
        let parsed = HttpRequestParser::new(req_str).parse();
        assert_eq!(Ok(expected_req), parsed);
    }
}
