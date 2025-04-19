use crate::test_helpers::test_repo::TestRepo;
use anyhow::{anyhow, Result};
use git_partial::core::metadata::RepositoryMetadata; // Use crate name 'git_partial'
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command; // Import the test helper

// Helper function to run the gitpartial command in a specific directory
// (Copied from clone_tests.rs, consider moving to a shared test_helpers module)
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

// Helper function to set up a basic partially cloned repo
fn setup_partial_repo(initial_paths: &[&str]) -> Result<(TestRepo, tempfile::TempDir, PathBuf)> {
    // 1. Set up a source Git repository
    let source_repo = TestRepo::new()?;
    source_repo.write_file("README.md", "# Main Readme")?;
    source_repo.write_file("src/core.rs", "// Core lib")?;
    source_repo.write_file("src/utils.rs", "// Utils lib")?;
    source_repo.write_file("docs/guide.md", "User guide")?;
    source_repo.write_file("data/data.txt", "important data")?;
    source_repo.add_all()?;
    source_repo.commit("Initial commit")?;
    let source_repo_url = source_repo.path_str()?;

    // 2. Clone it partially
    let clone_dir = tempfile::tempdir()?;
    let clone_path = clone_dir.path().to_path_buf();
    let clone_path_str = clone_path.to_string_lossy().to_string();
    let workspace_dir = PathBuf::from(".");

    let mut clone_args = vec!["clone", &source_repo_url, &clone_path_str, "--paths"];
    clone_args.extend(initial_paths);

    run_gitpartial(&workspace_dir, &clone_args)?;

    Ok((source_repo, clone_dir, clone_path))
}

#[test]
fn test_add_paths() -> Result<()> {
    // 1. Setup: Clone a repo with initial paths
    let initial_paths = ["README.md", "src/core.rs"];
    let (_source_repo, _clone_dir, clone_path) = setup_partial_repo(&initial_paths)?;

    // Verify initial state
    assert!(file_exists(&clone_path, "README.md"));
    assert!(file_exists(&clone_path, "src/core.rs"));
    assert!(!file_exists(&clone_path, "src/utils.rs"));
    assert!(!file_exists(&clone_path, "docs/guide.md"));
    assert!(file_exists(&clone_path, ".gitpartial/metadata.json"));

    let initial_metadata = RepositoryMetadata::load(&clone_path)?;
    let expected_initial_paths: HashSet<String> =
        initial_paths.iter().map(|s| s.to_string()).collect();
    assert_eq!(initial_metadata.checked_out_paths, expected_initial_paths);

    // 2. Action: Add new paths
    let paths_to_add = ["src/utils.rs", "docs/**"];
    // Note: add-paths runs IN the cloned repo directory
    run_gitpartial(&clone_path, &["add-paths", "src/utils.rs", "docs/**"])?;

    // 3. Verification
    // Check new files exist
    assert!(file_exists(&clone_path, "src/utils.rs"));
    assert!(file_exists(&clone_path, "docs/guide.md"));

    // Check original files still exist
    assert!(file_exists(&clone_path, "README.md"));
    assert!(file_exists(&clone_path, "src/core.rs"));

    // Check metadata is updated
    let updated_metadata = RepositoryMetadata::load(&clone_path)?;
    let mut expected_final_paths = expected_initial_paths;
    expected_final_paths.extend(paths_to_add.iter().map(|s| s.to_string()));
    assert_eq!(updated_metadata.checked_out_paths, expected_final_paths);
    // TODO: Check last_commit in metadata is updated if applicable (might need a remote change)

    Ok(())
}

#[test]
fn test_add_duplicate_paths() -> Result<()> {
    // 1. Setup: Clone a repo with initial paths
    let initial_paths = ["README.md", "src/**"];
    let (_source_repo, _clone_dir, clone_path) = setup_partial_repo(&initial_paths)?;

    // Verify initial state
    assert!(file_exists(&clone_path, "README.md"));
    assert!(file_exists(&clone_path, "src/core.rs"));
    assert!(file_exists(&clone_path, "src/utils.rs"));
    assert!(file_exists(&clone_path, ".gitpartial/metadata.json"));
    let initial_metadata = RepositoryMetadata::load(&clone_path)?;

    // 2. Action: Add a path that is already covered by a glob
    run_gitpartial(&clone_path, &["add-paths", "src/core.rs"])?;

    // 3. Verification
    let updated_metadata = RepositoryMetadata::load(&clone_path)?;
    // Metadata paths *will* change because we added the specific path src/core.rs
    let mut expected_after_specific_add = initial_metadata.checked_out_paths.clone();
    expected_after_specific_add.insert("src/core.rs".to_string()); // Expect specific path to be added
    assert_eq!(
        updated_metadata.checked_out_paths,
        expected_after_specific_add
    );

    // Action: Add the exact same glob again
    run_gitpartial(&clone_path, &["add-paths", "src/**"])?;
    let final_metadata = RepositoryMetadata::load(&clone_path)?;
    // Adding the same glob again should not change the paths set
    assert_eq!(
        final_metadata.checked_out_paths,
        updated_metadata.checked_out_paths
    );

    Ok(())
}
