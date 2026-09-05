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

use std::str::FromStr;
use std::{error::Error, fmt};

use crate::resources::Resource;
use crate::schemes::http::{HttpRequest, HttpVersion};
use crate::schemes::https::HttpsRequest;
use crate::schemes::{BuildResource, CreateRequest};

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
        let normalized_s = s.to_ascii_lowercase();

        let (raw_scheme, remainder) = normalized_s
            .split_once("://")
            .ok_or(UrlParseError::UnrecognizedStructure)?;
        let scheme = Scheme::from_str(raw_scheme)?;
        let default_port = match scheme {
            Scheme::Http => "80",
            Scheme::Https => "443",
        };
        let (authority, path) = remainder.split_once("/").unwrap_or((remainder, ""));
        let (hostname, port) = authority
            .split_once(":")
            .unwrap_or((authority, default_port));

        Ok(Url {
            scheme,
            host: hostname.to_ascii_lowercase(),
            port: port.parse().unwrap(),
            path: String::from("/") + path,
        })
    }
}

impl Url {
    pub fn request(&self) -> Result<Resource, Box<dyn Error>> {
        // Note - I initially tried to return `impl Response` here, but rust book 10.2 indicates
        // that we must return a single type. It doesn't give--at least to my eye--a satisfying
        // answer for why this is. We also can't utilize dynamic dispatch as-is because the FromStr
        // trait bound on Response is Sized, and Sized is not dyn compatible.
        //
        // What I'm doing instead is adding a Resource enum that is separate from the
        // request-response round-trip. This decouples the transport from the resource itself, and
        // gives us a layer to parse headers, etc. into useable data. Feels like excessive and leaky
        // abstraction but I have a hunch it may help avoid a combinatorial explosion down the road.
        match self.scheme {
            Scheme::Http => {
                let request = HttpRequest {
                    version: HttpVersion::Http10,
                    host: &self.host,
                    port: self.port,
                    path: &self.path,
                };
                let response = request.get()?;
                Ok(response.build_resource())
            }
            Scheme::Https => {
                let request = HttpsRequest {
                    version: HttpVersion::Http10,
                    host: &self.host,
                    port: self.port,
                    path: &self.path,
                };
                let response = request.get()?;
                Ok(response.build_resource())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
