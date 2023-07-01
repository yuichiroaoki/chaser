use std::error::Error;

use clap::{Parser, Subcommand};
mod checkpoint;
mod config;
mod import;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Check configuration
    Conf {
        /// Set configuration name
        #[clap(short, long, default_value = "default")]
        name: String,
    },
    /// Create a checkpoint
    Checkpoint {
        /// Checkpoint path
        #[clap(short, long)]
        path: String,
        /// Set configuration name
        #[clap(short, long, default_value = "default")]
        name: String,
    },
    /// Import pools from checkpoint
    Import {
        /// Checkpoint path
        #[clap(short, long)]
        path: String,
        /// Set configuration name
        #[clap(short, long, default_value = "default")]
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Conf { name } => {
            let conf = config::get_config(name);
            println!("{conf:#?}");
        }
        Commands::Checkpoint { name, path } => {
            checkpoint::create_checkpoint(name, path).await?;
        }
        Commands::Import { name, path } => {
            import::import_pool(name, path).await?;
        }
    }

    Ok(())
}
