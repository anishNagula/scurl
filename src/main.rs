mod cli;
mod request;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use request::perform_request;
use utils::init_logger;
use log::info;
use std::process;

fn main() {

    // initialize env_logger
    init_logger();

    let cli = Cli::parse();
    info!("Parsed CLI arguments successfully");

    let result = match &cli.command {
        Commands::Get { url, output , headers} => {
            perform_request("GET", url, None, output.as_deref(), headers)
        }
        Commands::Post { url, data, output , headers} => {
            perform_request("POST", url, data.as_deref(), output.as_deref(), headers)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}
