use anyhow::{Context, Result};
use log::{debug, info};
use std::fs;
use std::path::Path;

use crate::core::metadata::RepositoryMetadata;
use crate::git::commands;

/// Clone a repository with specified paths
pub async fn clone_repository(
    repo_url: &str,
    destination: &str,
    paths: &[String],
) -> Result<()> {
    info!(
        "Starting partial clone from {} to {}",
        repo_url, destination
    );
    debug!("Paths to include: {:?}", paths);

    let dest_path = Path::new(destination);

    // Check if destination exists and is not empty
    if dest_path.exists() {
        if fs::read_dir(dest_path)?.next().is_none() {
            // Directory exists but is empty, proceed
        } else {
            anyhow::bail!(
                "Destination directory '{}' exists and is not empty.",
                destination
            );
        }
    } else {
        // Create destination directory if it doesn't exist
        fs::create_dir_all(dest_path)
            .with_context(|| format!("Failed to create destination directory: {}", destination))?;
    }

    // Perform sparse clone into the destination directory
    commands::clone_sparse(repo_url, destination)
        .with_context(|| format!("Failed to perform sparse clone into {}", destination))?;

    // Set sparse-checkout paths within the cloned repository
    commands::set_sparse_checkout(dest_path, paths)
        .context("Failed to set sparse checkout paths")?;

    // Create and save metadata
    let mut metadata = RepositoryMetadata::new(repo_url.to_string());
    metadata.add_paths(paths);

    // Get the current HEAD commit and set it in metadata
    let head_commit = commands::get_head_commit(dest_path).context("Failed to get HEAD commit")?;
    metadata.set_last_commit(&head_commit);

    metadata
        .save(dest_path)
        .context("Failed to save metadata")?;

    info!("Partial clone completed in {}", destination);
    Ok(())
}
