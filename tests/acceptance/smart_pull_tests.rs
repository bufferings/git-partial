use crate::test_helpers::test_repo::TestRepo;
use anyhow::{anyhow, Result};
use git_partial::core::metadata::RepositoryMetadata;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

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

// Helper to check if a file exists relative to a base path
fn file_exists(
    base_path: &Path,
    relative_path: &str,
) -> bool {
    base_path.join(relative_path).exists()
}

// Helper to check file content
fn get_file_content(
    base_path: &Path,
    relative_path: &str,
) -> Result<String> {
    let file_path = base_path.join(relative_path);
    Ok(std::fs::read_to_string(file_path)?)
}

// Helper function to set up a source repo and a partial clone
// Returns the source repo helper and the path to the local clone directory
fn setup_repos_for_pull(initial_paths: &[&str]) -> Result<(TestRepo, TempDir, PathBuf)> {
    // 1. Source Repo Setup
    let source_repo = TestRepo::new()?;
    source_repo.write_file("README.md", "# Main Readme v1")?;
    source_repo.write_file("src/frontend/main.js", "// Frontend main v1")?;
    source_repo.write_file("src/frontend/button.js", "// Button v1")?;
    source_repo.write_file("src/backend/server.js", "// Backend server v1")?;
    source_repo.add_all()?;
    source_repo.commit("Initial commit")?;
    let source_repo_url = source_repo.path_str()?;

    // 2. Local Repo Setup (Partial Clone)
    let local_repo_tempdir = tempfile::tempdir()?; // Create an empty temp dir
    let local_repo_path = local_repo_tempdir.path().to_path_buf(); // Get its path
    let local_repo_path_str = local_repo_path.to_string_lossy().to_string();
    let workspace_dir = PathBuf::from(".");

    let mut clone_args = vec!["clone", &source_repo_url, &local_repo_path_str, "--paths"];
    clone_args.extend(initial_paths);

    // Clone into the empty temp dir
    run_gitpartial(&workspace_dir, &clone_args)?;

    Ok((source_repo, local_repo_tempdir, local_repo_path))
}

#[test]
fn test_smart_pull_updates_files() -> Result<()> {
    // 1. Setup
    let initial_paths = ["src/frontend/**", "README.md"];
    let (source_repo, _local_repo_dir, local_path) = setup_repos_for_pull(&initial_paths)?;
    // local_path is now PathBuf

    // 2. Modify source repo
    source_repo.write_file("README.md", "# Main Readme v2")?;
    source_repo.write_file("src/frontend/button.js", "// Button v2")?;
    source_repo.add_all()?;
    let commit2 = source_repo.commit("Update tracked files")?;

    // 3. Action: Run smart-pull in local repo
    run_gitpartial(&local_path, &["smart-pull"])?;

    // 4. Verification
    assert_eq!(
        get_file_content(&local_path, "README.md")?,
        "# Main Readme v2"
    );
    assert_eq!(
        get_file_content(&local_path, "src/frontend/button.js")?,
        "// Button v2"
    );
    assert!(file_exists(&local_path, "src/frontend/main.js")); // Should still exist
    assert!(!file_exists(&local_path, "src/backend/server.js")); // Should still not exist

    // Verify metadata commit updated
    let metadata = RepositoryMetadata::load(&local_path)?;
    assert_eq!(metadata.last_commit, Some(commit2));

    Ok(())
}

#[test]
fn test_smart_pull_new_files() -> Result<()> {
    // 1. Setup
    let initial_paths = ["src/frontend/**"];
    let (source_repo, _local_repo_dir, local_path) = setup_repos_for_pull(&initial_paths)?;

    // 2. Modify source repo: add new file within tracked path
    source_repo.write_file("src/frontend/new_component.js", "// New Component")?;
    source_repo.add_all()?;
    let commit2 = source_repo.commit("Add new frontend component")?;

    // 3. Action: Run smart-pull in local repo
    run_gitpartial(&local_path, &["smart-pull"])?;

    // 4. Verification
    assert!(file_exists(&local_path, "src/frontend/new_component.js"));
    assert_eq!(
        get_file_content(&local_path, "src/frontend/new_component.js")?,
        "// New Component"
    );
    assert!(file_exists(&local_path, "src/frontend/main.js")); // Should still exist

    // Verify metadata commit updated
    let metadata = RepositoryMetadata::load(&local_path)?;
    assert_eq!(metadata.last_commit, Some(commit2));

    Ok(())
}

#[test]
fn test_smart_pull_ignores_nonmatching_changes() -> Result<()> {
    // 1. Setup
    let initial_paths = ["src/frontend/**"];
    let (source_repo, _local_repo_dir, local_path) = setup_repos_for_pull(&initial_paths)?;
    let initial_commit = RepositoryMetadata::load(&local_path)?.last_commit;

    // 2. Modify source repo: add file outside tracked paths
    source_repo.write_file("src/backend/new_api.js", "// New API")?;
    source_repo.add_all()?;
    let commit2 = source_repo.commit("Add backend API")?;

    // 3. Action: Run smart-pull in local repo
    run_gitpartial(&local_path, &["smart-pull"])?;

    // 4. Verification
    // New backend file should NOT exist
    assert!(!file_exists(&local_path, "src/backend/new_api.js"));
    // Existing tracked files should remain unchanged
    assert!(file_exists(&local_path, "src/frontend/main.js"));
    assert_eq!(
        get_file_content(&local_path, "src/frontend/main.js")?,
        "// Frontend main v1"
    );

    // Verify metadata commit updated (even if no files changed locally)
    let metadata = RepositoryMetadata::load(&local_path)?;
    assert_eq!(metadata.last_commit, Some(commit2)); // Commit should update
    assert_ne!(metadata.last_commit, initial_commit);

    Ok(())
}
