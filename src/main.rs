use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;

mod cli;
mod core;
mod git;
mod remote;
mod utils;

/// GitPartial - A tool for efficiently working with large Git repositories
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Clone only part of a repository
    Clone {
        /// Repository URL to clone
        repo_url: String,

        /// Destination directory for the clone
        destination: String,

        /// Paths to include in the partial clone
        #[clap(long, value_parser, num_args = 1.., value_delimiter = ' ')]
        paths: Vec<String>,
    },

    /// Add new paths to the partial checkout
    AddPaths {
        /// New paths to include in the checkout
        #[clap(value_parser, num_args = 1.., value_delimiter = ' ')]
        paths: Vec<String>,
    },

    /// Show status of the partial checkout
    Status,

    /// Pull only changes relevant to the checked-out paths
    SmartPull,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    info!("GitPartial starting...");

    match cli.command {
        Commands::Clone {
            repo_url,
            destination,
            paths,
        } => {
            println!(
                "Cloning repository: {} to {} with paths: {:?}",
                repo_url, destination, paths
            );
            cli::clone::clone_repository(&repo_url, &destination, &paths).await?;
        }
        Commands::AddPaths { paths } => {
            println!("Adding paths: {:?}", paths);
            cli::add_paths::add_new_paths(&paths).await?;
        }
        Commands::Status => {
            println!("Status:");
            let status = cli::status::show_status().await?;
            println!("{}", status);
        }
        Commands::SmartPull => {
            println!("Smart pulling changes...");
            cli::smart_pull::perform_smart_pull().await?;
        }
    }

    Ok(())
}
