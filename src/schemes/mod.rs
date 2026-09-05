use crate::resources::Resource;
use std::{error::Error, fmt, str::FromStr};

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

pub trait CreateRequest {
    fn get(&self) -> Result<impl BuildResource, Box<dyn Error>>;
}

pub trait BuildResource: FromStr {
    fn build_resource(&self) -> Resource;
}

pub mod http;
pub mod https;
