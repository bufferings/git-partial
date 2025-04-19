use anyhow::{Context, Result};
use log::info;
use std::env;

use crate::core::metadata::RepositoryMetadata;
use crate::git::commands;
use crate::git::sparse;

/// Smart pull updates only the checked-out paths
pub async fn perform_smart_pull() -> Result<()> {
    info!("Starting smart pull");

    // Check if repo is using sparse checkout
    if !sparse::is_sparse_checkout()? {
        anyhow::bail!(
            "This repository is not using sparse checkout. Did you clone it with git-partial?"
        );
    }

    // Fetch latest changes
    info!("Fetching latest changes");
    commands::run_git_command(&["fetch", "origin"]).context("Failed to fetch changes")?;

    // Get current branch
    let current_branch = commands::run_git_command(&["branch", "--show-current"])
        .context("Failed to get current branch")?
        .trim()
        .to_string();

    info!("Current branch: {}", current_branch);

    // Perform a merge-based pull optimized for sparse checkout
    commands::run_git_command(&["merge", "--ff-only", &format!("origin/{}", current_branch)])
        .context("Failed to perform smart pull")?;

    // After successful pull, update the metadata
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let mut metadata = RepositoryMetadata::load(&current_dir).context("Failed to load metadata")?;

    let head_commit = commands::get_head_commit(&current_dir)
        .context("Failed to get new HEAD commit after pull")?;
    metadata.set_last_commit(&head_commit);

    metadata
        .save(&current_dir)
        .context("Failed to save updated metadata after pull")?;

    info!("Smart pull completed successfully and metadata updated");
    Ok(())
}
