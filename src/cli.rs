use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "scurl", version, about = "Fetch web pages or APIs (GET, POST) easily")]
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
    },
    Post {
        url: String,

        #[arg(short, long)]
        data: Option<String>,

        #[arg(short, long)]
        output: Option<String>,

        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
    },
}
