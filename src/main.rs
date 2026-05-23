#!warn[(clippy:all, clippy:pedantic)]

use std::env;
use std::error::Error;
use std::str::FromStr;
use tofino::url::Url;

#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), Box<dyn Error>> {
    let raw_url = env::args().next_back().unwrap();
    let url = Url::from_str(&raw_url)?;
    println!("{:#?}", url);
    Ok(())
}
