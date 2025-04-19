use anyhow::{Context, Result};
use log::info;
use std::env;

use crate::core::metadata::RepositoryMetadata;
use crate::git::commands;
use crate::git::sparse;

/// Display status information about the partial checkout
pub async fn show_status() -> Result<String> {
    info!("Checking partial checkout status");
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Load metadata first to check if it's a git-partial repo
    let metadata = match RepositoryMetadata::load(&current_dir) {
        Ok(meta) => meta,
        Err(_) => {
            return Ok(
                "Current directory is not a git-partial repository (metadata not found)."
                    .to_string(),
            );
        }
    };

    // Check if repo is using sparse checkout (redundant if metadata loaded, but good sanity check)
    if !sparse::is_sparse_checkout()? {
        return Ok(
            "Warning: Repository metadata found, but sparse checkout is not enabled.".to_string(),
        );
    }

    // Fetch latest changes quietly
    info!("Fetching remote changes for status check...");
    commands::run_git_command_in_dir(&current_dir, &["fetch", "origin", "--quiet"])
        .context("Failed to fetch remote changes")?;

    // Get local and remote HEAD commit SHAs
    let local_commit = metadata
        .last_commit
        .clone()
        .unwrap_or_else(|| "<unknown>".to_string());
    let current_branch =
        commands::run_git_command_in_dir(&current_dir, &["branch", "--show-current"])
            .context("Failed to get current branch")?
            .trim()
            .to_string();

    let remote_commit_res = commands::run_git_command_in_dir(
        &current_dir,
        &["rev-parse", &format!("origin/{}", current_branch)],
    );

    let remote_status = match remote_commit_res {
        Ok(remote_commit) if remote_commit == local_commit => "Up-to-date".to_string(),
        Ok(remote_commit) => {
            // Check if local commit is an ancestor of remote commit
            match commands::run_git_command_in_dir(
                &current_dir,
                &["merge-base", "--is-ancestor", &local_commit, &remote_commit],
            ) {
                Ok(_) => format!(
                    "Behind remote ({} -> {})",
                    &local_commit[..7],
                    &remote_commit[..7]
                ),
                Err(_) => format!(
                    "Diverged from remote (local: {}, remote: {})",
                    &local_commit[..7],
                    &remote_commit[..7]
                ),
            }
        }
        Err(_) => format!(
            "Could not determine remote status for branch '{}'",
            current_branch
        ),
    };

    // Get git status --short
    let git_status = commands::run_git_command_in_dir(&current_dir, &["status", "--short"])
        .context("Failed to get git status")?;

    // Format output
    let mut output = String::new();
    output.push_str("Git Partial Status\n");
    output.push_str("=================\n\n");
    output.push_str(&format!("Branch: {} ({})\n", current_branch, remote_status));
    output.push_str(&format!("Last Synced Commit: {}\n", local_commit));
    output.push_str(&format!("Remote URL: {}\n\n", metadata.remote_url));

    output.push_str("Sparse checkout paths:\n");
    for path in &metadata.checked_out_paths {
        output.push_str(&format!("  - {}\n", path));
    }

    output.push_str("\nLocal changes:\n");
    if git_status.trim().is_empty() {
        output.push_str("  No changes\n");
    } else {
        for line in git_status.lines() {
            output.push_str(&format!("  {}\n", line));
        }
    }

    info!("Status check completed");
    Ok(output)
}
