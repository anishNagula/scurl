mod cli;
mod request;

use clap::Parser;
use cli::{Cli, Commands};
use request::perform_request;
use std::process;

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Get { url } => perform_request("GET", url, None),
        Commands::Post { url, data } => perform_request("POST", url, data.as_deref()),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
