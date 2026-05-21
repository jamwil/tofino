#!warn[(clippy:all, clippy:pedantic)]

use std::env;
use std::process;
use std::str::FromStr;
use tofino::url::Url;

#[cfg(not(tarpaulin_include))]
fn main() {
    let raw_url = env::args().next_back().unwrap();
    if let Ok(url) = Url::from_str(&raw_url.to_ascii_lowercase()) {
        println!("Successfully parsed {:#?}", url);
    } else {
        eprintln!("Couldn't parse URL.");
        process::exit(1);
    }
}
