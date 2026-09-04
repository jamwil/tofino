use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display},
    io::{Read, Write},
    net::TcpStream,
    str::FromStr,
};

use crate::schemes::{Request, Response};

#[derive(Debug)]
pub enum ResponseParseError {
    EmptyResponse,
    BadVersion(String),
    BadStatus(String),
    BadExplanation(String),
    BadHeaders,
    BadHeader(String),
}

impl Error for ResponseParseError {}

impl fmt::Display for ResponseParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseParseError::EmptyResponse => write!(f, "Received an empty response"),
            ResponseParseError::BadVersion(s) => {
                write!(f, "Could not interpret HTTP version: {}", s)
            }
            ResponseParseError::BadStatus(s) => write!(
                f,
                "Could not interpret HTTP status code from status line: {}",
                s
            ),
            ResponseParseError::BadExplanation(s) => {
                write!(
                    f,
                    "Could not interpret HTTP explanation from status line: {}",
                    s
                )
            }
            ResponseParseError::BadHeaders => write!(f, "Could not parse HTTP response headers"),
            ResponseParseError::BadHeader(s) => write!(f, "Could not parse header: {}", s),
        }
    }
}

pub enum HttpVersion {
    Http10,
    Http11,
}

impl FromStr for HttpVersion {
    type Err = ResponseParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "HTTP/1.0" => Ok(Self::Http10),
            "HTTP/1.1" => Ok(Self::Http11),
            v => Err(ResponseParseError::BadVersion(v.to_owned()))?,
        }
    }
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpVersion::Http10 => write!(f, "HTTP/1.0"),
            HttpVersion::Http11 => write!(f, "HTTP/1.1"),
        }
    }
}

pub struct HttpRequest<'a> {
    pub version: HttpVersion,
    pub host: &'a str,
    pub port: usize,
    pub path: &'a str,
}

impl<'a> Request for HttpRequest<'a> {
    fn get(&self) -> Result<impl Response, Box<dyn Error>> {
        // Construct the request
        let socket_addr = self.host.to_string() + ":" + &self.port.to_string();
        let request = format!(
            "GET {} {}\r\nHost: {}\r\n\r\n",
            &self.path, &self.version, &self.host
        );

        // Create a TCP socket connection and send the request
        let mut buffer = String::new();
        let mut stream = TcpStream::connect(socket_addr)?;
        stream.write_all(&request.into_bytes())?;
        stream.read_to_string(&mut buffer)?;

        // Parse the response or propogate the error
        Ok(HttpResponse::from_str(&buffer)?)
    }
}

pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: String,
    pub explanation: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Response for HttpResponse {}

impl FromStr for HttpResponse {
    type Err = ResponseParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let version: HttpVersion;
        let status: String;
        let explanation: String;
        let mut headers: HashMap<String, String> = HashMap::new();
        let body: String;

        // Parse the statusline
        if let Some(statusline) = lines.next() {
            let mut statusline_split = statusline.splitn(3, " ");
            version = statusline_split.next().unwrap().parse()?;
            status = statusline_split
                .next()
                .ok_or(ResponseParseError::BadStatus(statusline.to_owned()))?
                .to_ascii_uppercase();
            explanation = statusline_split
                .next()
                .ok_or(ResponseParseError::BadExplanation(statusline.to_owned()))?
                .to_ascii_uppercase();
        } else {
            return Err(ResponseParseError::EmptyResponse);
        }

        // Parse the headers
        loop {
            if let Some(headerline_candidate) = lines.next() {
                // If it's an empty line, we're finished with headers; the next line will be the body
                if headerline_candidate.is_empty() {
                    break;
                }

                // Try to split the header
                let (key, value) =
                    headerline_candidate
                        .split_once(": ")
                        .ok_or(ResponseParseError::BadHeader(
                            headerline_candidate.to_owned(),
                        ))?;

                // Put it in the bank (and preserve case for header values)
                headers.insert(key.to_ascii_lowercase(), value.to_owned());
            } else {
                return Err(ResponseParseError::BadHeaders);
            }
        }

        // Parse the body (nested scope is for consistency)
        {
            body = lines.collect();
        }

        Ok(HttpResponse {
            version,
            status,
            explanation,
            headers,
            body,
        })
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_response() {
        let response = "";
        let result = HttpResponse::from_str(response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::EmptyResponse)
                && e.to_string() == "Received an empty response")
        );
    }

    #[test]
    fn parse_response_with_bad_version() {
        let statusline = String::from("HTTP/2.0 200 OK");
        let response = statusline + "\r\nContent-Type: text/html\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadVersion(_))
                && e.to_string() == "Could not interpret HTTP version: HTTP/2.0")
        );
    }

    #[test]
    fn parse_response_with_bad_status_code() {
        let statusline = String::from("HTTP/1.0");
        let response = statusline + "\r\nContent-Type: text/html\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadStatus(_))
                && e.to_string()
                    == "Could not interpret HTTP status code from status line: HTTP/1.0")
        );
    }

    #[test]
    fn parse_response_with_bad_explanation() {
        let statusline = String::from("HTTP/1.0 200");
        let response = statusline + "\r\nContent-Type: text/html\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadExplanation(_))
                && e.to_string()
                    == "Could not interpret HTTP explanation from status line: HTTP/1.0 200")
        );
    }

    #[test]
    fn parse_response_with_bad_headers() {
        let statusline = String::from("HTTP/1.0 200 OK");
        let response = statusline + "\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadHeaders)
                && e.to_string() == "Could not parse HTTP response headers")
        );
    }

    #[test]
    fn parse_response_with_bad_header() {
        let statusline = String::from("HTTP/1.0 200 OK");
        let response = statusline + "\r\nContent-Type; text/html\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadHeader(_))
                && e.to_string() == "Could not parse header: Content-Type; text/html")
        );
    }
}
