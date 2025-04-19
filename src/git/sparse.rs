use anyhow::{Context, Result};
use log::{debug, info};
use std::env;
use std::path::Path;

use crate::git::commands;

/// Clone a repository with sparse checkout
#[allow(dead_code)]
pub fn clone_sparse(
    repo_url: &str,
    target_path: &Path,
) -> Result<()> {
    info!(
        "Cloning repository with sparse checkout: {} to {:?}",
        repo_url, target_path
    );

    // Create target directory if it doesn't exist
    if !target_path.exists() {
        std::fs::create_dir_all(target_path)
            .with_context(|| format!("Failed to create directory: {:?}", target_path))?;
    }

    // Change to target directory and clone
    commands::run_git_command_in_dir(
        target_path,
        &["clone", "--filter=blob:none", "--sparse", repo_url, "."],
    )?;

    // Enable sparse checkout
    commands::run_git_command_in_dir(target_path, &["config", "core.sparseCheckout", "true"])?;

    Ok(())
}

/// Set sparse checkout paths
#[allow(dead_code)]
pub fn set_sparse_paths(
    repo_path: &Path,
    paths: &[String],
) -> Result<()> {
    info!(
        "Setting sparse checkout paths: {:?} in {:?}",
        paths, repo_path
    );

    let paths_str: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    commands::run_git_command_in_dir(repo_path, &["sparse-checkout", "set", &paths_str.join(" ")])?;

    Ok(())
}

/// Add paths to sparse checkout
#[allow(dead_code)]
pub fn add_paths(paths: &[String]) -> Result<()> {
    info!("Adding paths to sparse checkout: {:?}", paths);
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    commands::set_sparse_checkout(&current_dir, paths)?;
    Ok(())
}

/// Get current sparse checkout paths
#[allow(dead_code)]
pub fn get_current_paths() -> Result<Vec<String>> {
    let output = commands::run_git_command(&["sparse-checkout", "list"])?;
    let paths: Vec<String> = output
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    debug!("Current sparse checkout paths: {:?}", paths);
    Ok(paths)
}

/// Check if the repository is using sparse checkout
pub fn is_sparse_checkout() -> Result<bool> {
    let git_dir = Path::new(".git");
    if !git_dir.exists() {
        return Ok(false);
    }

    let output = commands::run_git_command(&["config", "core.sparseCheckout"])?;
    Ok(output.trim() == "true")
}
