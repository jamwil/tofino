#!warn[(clippy:all, clippy:pedantic)]

// Parts that vary:
// - scheme (http, https, data, file, etc.)
// - http version (secondary if scheme is http/https)
//
// A url consists of scheme://hostname/path
//                   [    ]   [      ][   ]

use std::env;
use std::process;
use std::str::FromStr;

#[derive(Debug)]
enum Scheme {
    Http,
    Https,
}

#[derive(Debug)]
struct UnsupportedSchemeError;

impl FromStr for Scheme {
    type Err = UnsupportedSchemeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Scheme::Http),
            "https" => Ok(Scheme::Https),
            _ => Err(UnsupportedSchemeError),
        }
    }
}

#[derive(Debug)]
struct Url {
    scheme: Scheme,
    hostname: String,
    path: String,
}

#[derive(Debug)]
struct UrlParseError;

impl FromStr for Url {
    type Err = UrlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Append an empty path if none is specified
        let mut normalized_s = s.to_owned();
        if !normalized_s.ends_with("/") {
            normalized_s = normalized_s + "/";
        }

        // Determine the scheme
        let (raw_scheme, remainder) = normalized_s.split_once("://").ok_or(UrlParseError)?;
        let scheme = Scheme::from_str(raw_scheme).map_err(|_| UrlParseError)?;

        // Determine the hostname and path
        let (hostname, path) = remainder.split_once("/").ok_or(UrlParseError)?;

        Ok(Url {
            scheme: scheme,
            hostname: hostname.to_owned(),
            path: String::from("/") + path,
        })
    }
}

fn main() {
    let raw_url = env::args().last().unwrap();
    if let Ok(url) = Url::from_str(&raw_url.to_ascii_lowercase()) {
        println!("Successfully parsed {:#?}", url);
    } else {
        eprintln!("Couldn't parse URL.");
        process::exit(1);
    }
}
