#!warn[(clippy:all, clippy:pedantic)]

//! # Tofino
//!
//! **The fingerpaint version of a web browser.**
//!
//! Use it like this:
//!
//! ```
//! use std::error::Error;
//! use std::str::FromStr;
//! use tofino::url::Url;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!   // We'll fake the raw url but you could retrieve it from the cli args like so:
//!   // let raw_url = std::env::args().next_back().unwrap();
//!   let raw_url = "http://example.org/";
//!
//!   let url = Url::from_str(&raw_url)?;
//!
//!   // Do something fun, more fun than this I hope
//!   println!("{:#?}", url);
//!
//!   Ok(())
//! }
//! ```

pub mod cli;
pub mod config;
pub mod html;
pub mod resources;
pub mod schemes;
pub mod url;
