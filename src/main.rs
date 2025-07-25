mod cli;
mod request;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use request::perform_request;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().expect("Failed to start tokio runtime");

    let cli = Cli::parse();

    rt.block_on(async {
        match  &cli.command {
            Commands::Get { url, output, headers, verbose } => {
                if let Err(e) = perform_request("GET", url, None, output.as_deref(), headers, *verbose).await {
                    eprintln!("Error: {}", e);
                }
            }
            Commands::Post { url, data, output, headers, verbose } => {
                if let Err(e) = perform_request("POST", url, data.as_deref(), output.as_deref(), headers, *verbose).await {
                    eprintln!("Error: {}", e);
                }
            }
        }
    });
}