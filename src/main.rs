mod cli;
mod request;

use clap::Parser;
use cli::{Cli, Commands};
use request::perform_request;
use std::process;

fn main() {
    let cli = Cli::parse();

    let res = match &cli.command {
        Commands::Get { url, output, headers } => perform_request("GET", url, None, output.as_deref(), headers),
        Commands::Post { url, data, output, headers } => perform_request("POST", url, data.as_deref(), output.as_deref(), headers),
        Commands::Put { url, data, output, headers } => perform_request("PUT", url, data.as_deref(), output.as_deref(), headers),
        Commands::Delete { url, output, headers } => perform_request("DELETE", url, None, output.as_deref(), headers),
    };

    if let Err(e) = res {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
