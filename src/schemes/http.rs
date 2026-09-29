use super::{BuildResource, CreateRequest, ResponseParseError};
use crate::resources::{Resource, WebResource};
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display},
    io::{Read, Write},
    net::TcpStream,
    str::FromStr,
};

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

impl<'a> CreateRequest for HttpRequest<'a> {
    fn get(&self) -> Result<impl BuildResource, Box<dyn Error>> {
        // Construct the request
        let socket_addr = self.host.to_string() + ":" + &self.port.to_string();
        let request = format!(
            "GET {} {}\r\nHost: {}\r\n\r\n",
            self.path, self.version, self.host
        );

        // Create a TCP socket connection and send the request
        let mut buffer = String::new();
        let mut stream = TcpStream::connect(socket_addr)?;
        stream.write_all(&request.into_bytes())?;
        stream.read_to_string(&mut buffer)?;

        // Parse the response or propagate the error
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

impl FromStr for HttpResponse {
    type Err = ResponseParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let version: HttpVersion;
        let status: String;
        let explanation: String;
        let mut headers: HashMap<String, String> = HashMap::new();
        let body: String;

        // Parse the status_line
        if let Some(status_line) = lines.next() {
            let mut status_line_split = status_line.splitn(3, " ");
            version = status_line_split.next().unwrap().parse()?;
            status = status_line_split
                .next()
                .ok_or(ResponseParseError::BadStatus(status_line.to_owned()))?
                .to_ascii_uppercase();
            explanation = status_line_split
                .next()
                .ok_or(ResponseParseError::BadExplanation(status_line.to_owned()))?
                .to_ascii_uppercase();
        } else {
            return Err(ResponseParseError::EmptyResponse);
        }

        // Parse the headers
        loop {
            if let Some(header_line_candidate) = lines.next() {
                // If it's an empty line, we're finished with headers; the next line will be the
                // body
                if header_line_candidate.is_empty() {
                    break;
                }

                // Try to split the header
                let (key, value) =
                    header_line_candidate
                        .split_once(": ")
                        .ok_or(ResponseParseError::BadHeader(
                            header_line_candidate.to_owned(),
                        ))?;

                // Put it in the bank (and preserve case for header values)
                headers.insert(key.to_ascii_lowercase(), value.to_owned());
            } else {
                return Err(ResponseParseError::BadHeaders);
            }
        }

        // Parse the body (nested scope is for consistency)
        {
            let remaining_lines: Vec<&str> = lines.collect();
            body = remaining_lines.join("\n");
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

impl BuildResource for HttpResponse {
    fn build_resource(&self) -> Resource {
        Resource::Web(WebResource {
            response_status: self.status.clone(),
            response_explanation: self.explanation.clone(),
            response_headers: self.headers.clone(),
            response_body: self.body.clone(),
        })
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
        let status_line = String::from("HTTP/2.0 200 OK");
        let response = status_line + "\r\nContent-Type: text/html\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadVersion(_))
                && e.to_string() == "Could not interpret HTTP version: HTTP/2.0")
        );
    }

    #[test]
    fn parse_response_with_bad_status_code() {
        let status_line = String::from("HTTP/1.0");
        let response = status_line + "\r\nContent-Type: text/html\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadStatus(_))
                && e.to_string()
                    == "Could not interpret HTTP status code from status line: HTTP/1.0")
        );
    }

    #[test]
    fn parse_response_with_bad_explanation() {
        let status_line = String::from("HTTP/1.0 200");
        let response = status_line + "\r\nContent-Type: text/html\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadExplanation(_))
                && e.to_string()
                    == "Could not interpret HTTP explanation from status line: HTTP/1.0 200")
        );
    }

    #[test]
    fn parse_response_with_bad_headers() {
        let status_line = String::from("HTTP/1.0 200 OK");
        let response = status_line + "\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadHeaders)
                && e.to_string() == "Could not parse HTTP response headers")
        );
    }

    #[test]
    fn parse_response_with_bad_header() {
        let status_line = String::from("HTTP/1.0 200 OK");
        let response = status_line + "\r\nContent-Type; text/html\r\n";
        let result = HttpResponse::from_str(&response);
        assert!(
            result.is_err_and(|e| matches!(e, ResponseParseError::BadHeader(_))
                && e.to_string() == "Could not parse header: Content-Type; text/html")
        );
    }
}
