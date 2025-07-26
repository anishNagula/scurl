use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "scurl", version, about = "Minimal curl clone in Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Get {
        url: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
        #[arg(short, long)]
        verbose: bool,
    },
    Post {
        url: String,
        #[arg(short, long)]
        data: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
        #[arg(short, long)]
        verbose: bool,
    },
    Head {
        url: String,
         #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
        #[arg(short, long)]
        verbose: bool,
    }
}