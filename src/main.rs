mod cli;
mod request;

use clap::Parser;
use cli::{Cli, Commands};
use request::perform_request;
use std::process;

fn main() {
    let cli = Cli::parse();

    let res = match &cli.command {
        Commands::Get { url, output } => perform_request("GET", url, None, output.as_deref()),
        Commands::Post { url, data, output } => perform_request("POST", url, data.as_deref(), output.as_deref()),
    };

    if let Err(e) = res {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
