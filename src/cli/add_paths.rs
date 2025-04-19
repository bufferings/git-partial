use anyhow::{Context, Result};
use log::{debug, info};
use std::env;

use crate::core::metadata::RepositoryMetadata;
use crate::git::commands;
use crate::git::sparse;

/// Add new paths to the sparse checkout
pub async fn add_new_paths(paths: &[String]) -> Result<()> {
    info!("Adding new paths to sparse checkout");
    debug!("New paths: {:?}", paths);

    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Load existing metadata
    let mut metadata = RepositoryMetadata::load(&current_dir).context("Failed to load metadata")?;

    // Check if repo is using sparse checkout (can be simplified by checking metadata)
    if !sparse::is_sparse_checkout()? {
        anyhow::bail!(
            "This repository is not using sparse checkout. Did you clone it with git-partial?"
        );
    }

    // Determine the full set of paths (existing + new)
    let mut final_paths = metadata.checked_out_paths.clone();
    let mut added_new = false;
    for path in paths {
        if final_paths.insert(path.clone()) {
            added_new = true;
        }
    }

    // Only update sparse checkout and metadata if new paths were actually added
    if added_new {
        let final_paths_vec: Vec<String> = final_paths.iter().cloned().collect();

        // Set updated paths in sparse-checkout
        commands::set_sparse_checkout(&current_dir, &final_paths_vec)
            .context("Failed to update sparse checkout paths")?;

        // Update metadata object
        metadata.checked_out_paths = final_paths;
        // Optionally update last commit if needed, though add-paths might not change it

        // Save updated metadata
        metadata
            .save(&current_dir)
            .context("Failed to save updated metadata")?;

        info!("Successfully added new paths and updated metadata");
    } else {
        info!("No new paths to add. Sparse checkout and metadata remain unchanged.");
    }

    Ok(())
}
