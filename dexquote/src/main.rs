use clap::{Parser, Subcommand};
use std::error::Error;
mod checkpoint;
mod cli;
mod config;
mod import;
mod sync;

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
        /// Sync pools before import
        #[clap(short, long, default_value = "false")]
        sync: bool,
    },
    /// Update pool states
    Sync {
        /// Number of threads
        /// Default: 4
        #[clap(short, long, default_value = "4")]
        threads: usize,
        /// Set configuration name
        #[clap(short, long, default_value = "default")]
        name: String,
    },
    /// Show possible paths
    Path {
        #[clap(long)]
        token_in: String,
        #[clap(long)]
        token_out: String,
        #[clap(long, default_value = "1")]
        hop: u64,
        #[clap(short, long, default_value = "5")]
        path_result_limit: u64,
        /// Set configuration name
        #[clap(short, long, default_value = "default")]
        name: String,
    },
    /// Quote prices
    Quote {
        #[clap(long)]
        token_in: String,
        #[clap(long)]
        token_out: String,
        #[clap(short, long)]
        amount_in: String,
        #[clap(long, default_value = "1")]
        hop: u64,
        #[clap(short, long, default_value = "5")]
        path_result_limit: u64,
        /// Set configuration name
        #[clap(short, long, default_value = "default")]
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    match args.command {
        Commands::Conf { name } => {
            let conf = config::get_config(name);
            println!("{conf:#?}");
        }
        Commands::Checkpoint { name, path } => {
            checkpoint::create_checkpoint(name, path).await?;
        }
        Commands::Import { name, path, sync } => {
            import::import_pool(name, path, sync).await?;
        }
        Commands::Sync { threads, name } => {
            sync::update_pool_states(threads, name).await?;
        }
        Commands::Path {
            token_in,
            token_out,
            hop,
            path_result_limit,
            name,
        } => cli::path::show_paths(token_in, token_out, hop, path_result_limit, name).await,
        Commands::Quote {
            token_in,
            token_out,
            amount_in,
            hop,
            path_result_limit,
            name,
        } => {
            cli::path::show_best_prices(
                token_in,
                token_out,
                hop,
                path_result_limit,
                amount_in,
                name,
            )
            .await
        }
    }

    Ok(())
}
