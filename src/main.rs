#!warn[(clippy:all, clippy:pedantic)]

use std::env;
use tofino::cli::cli;
use tofino::tui::tui;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.contains(&String::from("--tui")) {
        match tui(args) {
            Ok(()) => (),
            Err(error) => eprintln!("{}", error),
        }
    } else {
        match cli(args) {
            Ok(body) => println!("{}", body),
            Err(error) => eprintln!("{}", error),
        }
    }
}
