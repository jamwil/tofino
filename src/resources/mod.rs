use core::fmt;
use std::{collections::HashMap, path::PathBuf};

pub struct WebResource {
    pub response_status: String,
    pub response_explanation: String,
    pub response_headers: HashMap<String, String>,
    pub response_body: String,
}

pub struct LocalResource {
    pub file_path: PathBuf,
    pub body: String,
}

pub enum Resource {
    Web(WebResource),
    Local(LocalResource),
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Web(r) => write!(f, "{}", r.response_body),
            Self::Local(r) => write!(f, "{}", r.body),
        }
    }
}
