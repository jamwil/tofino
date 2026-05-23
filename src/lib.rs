#!warn[(clippy:all, clippy:pedantic)]
//! **The fingerpaint version of a web browser.**

/// Parses a URL into a normalized data structure for downstream marshalling.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use tofino::url::{Url, Scheme};
///
/// let url = Url::from_str("http://example.org").unwrap();
/// assert_eq!(url.scheme, Scheme::Http);
/// assert_eq!(url.hostname, "example.org");
/// assert_eq!(url.path, "/");
/// ```
///
/// # Architecture
///
/// The first decision point when the browser receives a request to load a resource will be the
/// scheme, so we'll pivot the application entry point on that and provide semi-robust error
/// handling for determining how and where a provided URL might be malformed.
///
/// The resulting [`url::Url`] struct will be composed of a [`url::Scheme`] enum, along with `String` fields for
/// hostname and path.
///
/// # Errors
///
/// - [`url::UrlParseError`]
pub mod url {
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
        pub hostname: String,
        pub path: String,
    }

    impl FromStr for Url {
        type Err = UrlParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Append an empty path if none is specified
            let mut normalized_s = s.to_owned();
            if !normalized_s.ends_with("/") {
                normalized_s += "/";
            }

            // Determine the scheme, hostname and path
            let (raw_scheme, remainder) = normalized_s
                .split_once("://")
                .ok_or(UrlParseError::UnrecognizedStructure)?;
            let scheme = Scheme::from_str(raw_scheme)?;
            let (hostname, path) = remainder.split_once("/").unwrap();

            Ok(Url {
                scheme,
                hostname: hostname.to_owned(),
                path: String::from("/") + path,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_valid_http_url_with_inferred_root_path() {
            let url = Url::from_str("http://example.org").unwrap();
            assert_eq!(url.scheme, Scheme::Http);
            assert_eq!(url.hostname, "example.org");
            assert_eq!(url.path, "/");
        }

        #[test]
        fn parse_valid_https_url_with_inferred_root_path() {
            let url = Url::from_str("https://example.org").unwrap();
            assert_eq!(url.scheme, Scheme::Https);
            assert_eq!(url.hostname, "example.org");
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
}
