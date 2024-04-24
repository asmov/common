use std::process;
use clap::Parser;
use colored::Colorize;
use cargo_traitenum::cli;

fn main() {
    let cli = cli::Cli::parse();
    match cargo_traitenum::run(cli) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("{}{}", "[traitenum] ".red(), e);
            process::exit(1);
        }
    }
}
