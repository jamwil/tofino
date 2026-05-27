//! Parses a URL into a normalized data structure for downstream marshalling.
//!
//! # Examples
//!
//! ```
//! use std::str::FromStr;
//! use tofino::url::{Url, Scheme};
//!
//! let url = Url::from_str("http://example.org").unwrap();
//! assert_eq!(url.scheme, Scheme::Http);
//! assert_eq!(url.host, "example.org");
//! assert_eq!(url.path, "/");
//! ```
//!
//! # Architecture
//!
//! The first decision point when the browser receives a request to load a resource will be the
//! scheme, so we'll pivot the application entry point on that and provide semi-robust error
//! handling for determining how and where a provided URL might be malformed.
//!
//! The resulting [`Url`] struct will be composed of a [`Scheme`] enum, along with `String` fields for
//! hostname and path.
//!
//! # Errors
//!
//! - [`UrlParseError`]

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::FromStr;
use std::{error::Error, fmt};

/// Provides basic error details propogated from `Url::from_str`.
#[derive(Debug)]
pub enum UrlParseError {
    UnsupportedScheme,
    UnrecognizedStructure,
}

impl Error for UrlParseError {}

impl fmt::Display for UrlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlParseError::UnsupportedScheme => write!(f, "Unsupported URL scheme"),
            UrlParseError::UnrecognizedStructure => write!(f, "Unrecognized URL structure"),
        }
    }
}

/// The protocol or method that we'll use to interact with or display the resource.
#[derive(Debug, PartialEq)]
pub enum Scheme {
    Http,
    Https,
}

impl FromStr for Scheme {
    type Err = UrlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Scheme::Http),
            "https" => Ok(Scheme::Https),
            _ => Err(UrlParseError::UnsupportedScheme),
        }
    }
}

#[derive(Debug)]
pub enum ResponseParseError {
    EmptyResponse,
    BadVersion,
    BadStatus,
    BadExplanation,
    BadHeaders,
    BadHeader,
}

impl Error for ResponseParseError {}

impl fmt::Display for ResponseParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not parse the response")
    }
}

pub struct HttpResponse {
    pub version: String,
    pub status: String,
    pub explanation: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl FromStr for HttpResponse {
    type Err = ResponseParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let version: String;
        let status: String;
        let explanation: String;
        let mut headers: HashMap<String, String> = HashMap::new();
        let body: String;

        // Parse the statusline
        if let Some(statusline) = lines.next() {
            let mut statusline_split = statusline.splitn(3, " ");
            version = statusline_split
                .next()
                .ok_or(ResponseParseError::BadVersion)?
                .to_ascii_lowercase();
            status = statusline_split
                .next()
                .ok_or(ResponseParseError::BadStatus)?
                .to_ascii_lowercase();
            explanation = statusline_split
                .next()
                .ok_or(ResponseParseError::BadExplanation)?
                .to_ascii_lowercase();
        } else {
            return Err(ResponseParseError::EmptyResponse);
        }

        // Parse the headers
        loop {
            if let Some(headerline_candidate) = lines.next() {
                // If it's an empty line, we're finished with headers; the next line will be the body
                if headerline_candidate == "" {
                    break;
                }

                // Try to split the header
                let (key, value) = headerline_candidate
                    .split_once(": ")
                    .ok_or(ResponseParseError::BadHeader)?;

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

/// Represents a normalized `Url`.
///
/// ```text
/// scheme://hostname/path/
/// [....]   [......][....]
/// ```
///
/// When parsed using `std::str::FromStr` a trailing slash is added if not explicitly provided,
/// such that `http://example.org` and `http://example.org/` will resolve the same way.
///
/// See [`super::url`] for a usage example.
#[derive(Debug)]
pub struct Url {
    pub scheme: Scheme,
    pub host: String,
    pub port: usize,
    pub path: String,
}

impl FromStr for Url {
    type Err = UrlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Append an empty path if none is specified
        let mut normalized_s = s.to_ascii_lowercase();
        if !normalized_s.ends_with("/") {
            normalized_s += "/";
        }

        // Determine the scheme, hostname and path
        let (raw_scheme, remainder) = normalized_s
            .split_once("://")
            .ok_or(UrlParseError::UnrecognizedStructure)?;
        let scheme = Scheme::from_str(raw_scheme)?;
        let (authority, path) = remainder.split_once("/").unwrap();
        let (hostname, port) = authority.split_once(":").unwrap_or((authority, "80"));

        Ok(Url {
            scheme,
            host: hostname.to_ascii_lowercase(),
            port: port.parse().unwrap(),
            path: String::from("/") + path,
        })
    }
}

impl Url {
    pub fn request(&self) -> std::io::Result<HttpResponse> {
        // Construct the request, temorarily hardcoding port 80 for now
        let socket_addr = self.host.clone() + ":" + &self.port.to_string();
        let request = format!(
            "GET {} HTTP/1.0\r\nHost: {}\r\n\r\n",
            &self.path, &self.host
        );

        // Create a TCP socket connection and send the request
        let mut stream = TcpStream::connect(socket_addr)?;
        let mut buffer = String::new();
        stream.write_all(&request.into_bytes())?;
        stream.read_to_string(&mut buffer)?;
        let response = HttpResponse::from_str(&buffer).unwrap();

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[test]
    fn parse_valid_http_url_with_inferred_root_path() {
        let url = Url::from_str("http://example.org").unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host, "example.org");
        assert_eq!(url.path, "/");
    }

    #[test]
    fn parse_valid_http_url_with_explicit_port() {
        let url = Url::from_str("http://example.org:8080").unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host, "example.org");
        assert_eq!(url.port, 8080);
        assert_eq!(url.path, "/");
    }

    #[test]
    fn parse_valid_https_url_with_inferred_root_path() {
        let url = Url::from_str("https://example.org").unwrap();
        assert_eq!(url.scheme, Scheme::Https);
        assert_eq!(url.host, "example.org");
        assert_eq!(url.path, "/");
    }

    #[test]
    fn parse_url_with_invalid_scheme() {
        let result = Url::from_str("invalid://example.org");
        assert!(
            result.is_err_and(|e| matches!(e, UrlParseError::UnsupportedScheme)
                && e.to_string() == "Unsupported URL scheme")
        );
    }

    #[test]
    fn parse_url_with_unrecognized_structure() {
        let result = Url::from_str("http:example.org");
        assert!(
            result.is_err_and(|e| matches!(e, UrlParseError::UnrecognizedStructure)
                && e.to_string() == "Unrecognized URL structure")
        );
    }

    #[test]
    fn make_a_request() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method("GET").path("/");
            then.status(200)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body("hola");
        });

        let url = Url::from_str(&server.url("/")).unwrap();
        let response = url.request().unwrap();

        mock.assert();

        assert_eq!(response.status, "200");
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            "text/html; charset=UTF-8"
        );
        assert_eq!(response.body, "hola");
    }
}
