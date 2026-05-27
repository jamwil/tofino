#!warn[(clippy:all, clippy:pedantic)]

use std::env;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use tofino::url::Url;

#[derive(Debug)]
struct CliError;

impl Error for CliError {}

#[cfg(not(tarpaulin_include))]
impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tofino accepts a single URL argument")
    }
}

#[cfg(not(tarpaulin_include))]
fn cli() -> Result<(), Box<dyn Error>> {
    // Check that the program was invoked with two arguments
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        return Err(Box::new(CliError));
    }

    // Extract and display the normalized URL
    let raw_url = args.last().ok_or(CliError)?;
    let url = Url::from_str(raw_url)?;

    // Send a request and print the response
    let response = url.request()?;
    println!("{}", response.body);

    Ok(())
}

#[cfg(not(tarpaulin_include))]
fn main() {
    if let Err(e) = cli() {
        eprintln!("{}", e);
    }
}
