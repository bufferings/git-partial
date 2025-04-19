use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Run a git command and return the output
pub fn run_git_command(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

/// Run a git command in a specific directory and return the output
pub fn run_git_command_in_dir<P: AsRef<Path>>(
    dir: P,
    args: &[&str],
) -> Result<String> {
    let output = Command::new("git")
        .current_dir(dir.as_ref())
        .args(args)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

/// Clone a repository using sparse checkout
pub fn clone_sparse(
    repo_url: &str,
    destination: &str,
) -> Result<()> {
    // Use git clone with sparse checkout options again
    run_git_command(&[
        "clone",
        "--filter=blob:none",
        "--sparse",
        repo_url,
        destination,
    ])?;

    Ok(())
}

/// Set sparse checkout paths
pub fn set_sparse_checkout(
    repo_path: &Path,
    paths: &[String],
) -> Result<()> {
    // Prepend '/' to root-level files/dirs to avoid matching nested ones.
    // We only do this for paths without '/' or glob characters.
    let processed_paths: Vec<String> = paths
        .iter()
        .map(|p| {
            if !p.contains('/') && !p.contains('*') && !p.contains('?') && !p.contains('[') {
                format!("/{}", p)
            } else {
                p.clone()
            }
        })
        .collect();

    let paths_str: Vec<&str> = processed_paths.iter().map(|s| s.as_str()).collect();

    // Run sparse-checkout command in the repository directory
    let mut args = vec!["sparse-checkout", "set", "--no-cone", "--"];
    args.extend(paths_str);
    run_git_command_in_dir(repo_path, &args)?;

    // After setting paths, update the working directory using checkout
    // This seems to correctly remove files/dirs not matching the new patterns.
    run_git_command_in_dir(repo_path, &["checkout", "HEAD", "--force"])?;
    // run_git_command_in_dir(repo_path, &["rm", "-r", "--cached", "."])?;
    // run_git_command_in_dir(repo_path, &["reset", "--hard", "HEAD"])?;

    Ok(())
}

/// Get the current HEAD commit SHA
pub fn get_head_commit<P: AsRef<Path>>(repo_path: P) -> Result<String> {
    run_git_command_in_dir(repo_path, &["rev-parse", "HEAD"])
}
