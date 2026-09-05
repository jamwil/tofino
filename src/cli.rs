use crate::html;
use crate::url::Url;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct CliError;

impl Error for CliError {}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tofino accepts a single URL argument")
    }
}

pub fn cli(args: Vec<String>) -> Result<String, Box<dyn Error>> {
    // Check that the program was invoked with two arguments
    if args.len() != 2 {
        return Err(Box::new(CliError));
    }

    // Extract and display the normalized URL
    let raw_url = args.last().ok_or(CliError)?;
    let url = Url::from_str(raw_url)?;

    // Send a request and return the response
    let resource = url.request()?;
    Ok(html::show(&resource.to_string()))
}
