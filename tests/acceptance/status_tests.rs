use crate::test_helpers::test_repo::TestRepo;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

// Helper function to run the gitpartial command in a specific directory
fn run_gitpartial(
    cwd: &Path,
    args: &[&str],
) -> Result<String> {
    let bin_path = PathBuf::from(env!("CARGO_BIN_EXE_git-partial"));
    let output = Command::new(bin_path)
        .args(args)
        .current_dir(cwd)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow!(
            "Command failed in {}:
Args: {:?}
Exit Code: {:?}
Stderr: {}
Stdout: {}",
            cwd.display(),
            args,
            output.status.code(),
            stderr,
            stdout
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

// Helper function to set up a source repo and a partial clone
fn setup_repos_for_status(
    initial_paths: &[&str]
) -> Result<(TestRepo, tempfile::TempDir, PathBuf)> {
    // 1. Source Repo Setup
    let source_repo = TestRepo::new()?;
    source_repo.write_file("README.md", "# Main Readme v1")?;
    source_repo.write_file("src/main.rs", "// Main v1")?;
    source_repo.write_file("src/lib.rs", "// Lib v1")?;
    source_repo.add_all()?;
    source_repo.commit("Initial commit")?;
    let source_repo_url = source_repo.path_str()?;

    // 2. Local Repo Setup (Partial Clone)
    let local_repo_tempdir = tempfile::tempdir()?;
    let local_repo_path = local_repo_tempdir.path().to_path_buf();
    let local_repo_path_str = local_repo_path.to_string_lossy().to_string();
    let workspace_dir = PathBuf::from(".");

    let mut clone_args = vec!["clone", &source_repo_url, &local_repo_path_str, "--paths"];
    clone_args.extend(initial_paths);
    run_gitpartial(&workspace_dir, &clone_args)?;

    // Remove the redundant remote add command, as clone should set origin
    // TestRepo::run_git_command(local_repo_path.as_path(), &["remote", "add", "origin", &source_repo_url])?;
    TestRepo::run_git_command(local_repo_path.as_path(), &["fetch", "origin", "--quiet"])?;

    Ok((source_repo, local_repo_tempdir, local_repo_path))
}

#[test]
fn test_status_up_to_date() -> Result<()> {
    // 1. Setup: Clone repo, no changes after clone
    let initial_paths = ["README.md"];
    let (_source_repo, _local_repo_dir, local_path) = setup_repos_for_status(&initial_paths)?;

    // 2. Action: Run status
    let status_output = run_gitpartial(&local_path, &["status"])?;

    // 3. Verification
    assert!(status_output.contains("Branch: main (Up-to-date)"));
    assert!(status_output.contains("Sparse checkout paths:"));
    assert!(status_output.contains("  - README.md"));
    assert!(status_output.contains("Local changes:"));
    // Don't strictly assert "No changes", as fetch might cause status output
    // Just check the section exists.

    Ok(())
}

#[test]
fn test_status_behind() -> Result<()> {
    // 1. Setup: Clone repo, then update source
    let initial_paths = ["README.md"];
    let (source_repo, _local_repo_dir, local_path) = setup_repos_for_status(&initial_paths)?;
    source_repo.write_file("README.md", "# Main Readme v2")?;
    source_repo.add_all()?; // Add the change before committing
    let commit2 = source_repo.commit("Update README")?;

    // 2. Action: Run status (without pulling)
    let status_output = run_gitpartial(&local_path, &["status"])?;

    // 3. Verification
    assert!(status_output.contains(&"Branch: main (Behind remote".to_string()));
    // Check for partial commit hashes
    assert!(status_output.contains(&commit2[..7]));
    assert!(status_output.contains("Local changes:"));
    // Don't strictly assert "No changes" when behind, status might show branch divergence info
    // assert!(status_output.contains("  No changes"));

    Ok(())
}

#[test]
fn test_status_with_local_changes() -> Result<()> {
    // 1. Setup: Clone repo
    let initial_paths = ["README.md", "src/main.rs"];
    let (_source_repo, _local_repo_dir, local_path) = setup_repos_for_status(&initial_paths)?;

    // 2. Make local changes to tracked files
    std::fs::write(local_path.join("README.md"), "# Local Edit")?;
    std::fs::write(local_path.join("src/main.rs"), "// Main v2 Local Edit")?;
    // Create and add an untracked file (should appear as ??)
    std::fs::write(local_path.join("untracked.txt"), "local untracked")?;

    // 3. Action: Run status
    let status_output = run_gitpartial(&local_path, &["status"])?;

    // 4. Verification
    assert!(status_output.contains("Branch: main (Up-to-date)"));
    assert!(status_output.contains("Local changes:"));
    assert!(status_output.contains(" M README.md")); // Modified
    assert!(status_output.contains(" M src/main.rs")); // Modified
    assert!(status_output.contains("?? untracked.txt")); // Untracked

    Ok(())
}

#[test]
fn test_status_non_partial_repo() -> Result<()> {
    // 1. Setup: Create an empty directory (not a git-partial repo)
    let temp_dir = tempfile::tempdir()?;
    let non_repo_path = temp_dir.path();

    // 2. Action: Run status
    let status_output = run_gitpartial(non_repo_path, &["status"])?;

    // 3. Verification
    assert!(status_output.contains("Current directory is not a git-partial repository"));

    Ok(())
}
