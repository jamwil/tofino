#!warn[(clippy:all, clippy:pedantic)]

use std::env;
use tofino::cli::cli;

#[cfg(not(tarpaulin_include))]
fn main() {
    let args: Vec<_> = env::args().collect();
    match cli(args) {
        Ok(body) => println!("{}", body),
        Err(error) => eprintln!("{}", error),
    }
}
