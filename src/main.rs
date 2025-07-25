mod cli;
mod request;

use clap::Parser;
use cli::{Cli, Commands};
use request::perform_request;
use reqwest::blocking::Client;
use std::process;

fn main() {
    let cli = Cli::parse();
    let client = Client::new();  // reuse same client

    let result = match &cli.command {
        Commands::Get { url, output, headers } => {
            perform_request(&client, "GET", url, None, output.as_deref(), headers)
        }
        Commands::Post { url, data, output, headers } => {
            perform_request(&client, "POST", url, data.as_deref(), output.as_deref(), headers)
        }
        Commands::Put { url, data, output, headers } => {
            perform_request(&client, "PUT", url, data.as_deref(), output.as_deref(), headers)
        }
        Commands::Delete { url, output, headers } => {
            perform_request(&client, "DELETE", url, None, output.as_deref(), headers)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
