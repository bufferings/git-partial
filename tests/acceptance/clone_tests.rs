use crate::test_helpers::test_repo::TestRepo;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
// Import the test helper

// Helper function to run the gitpartial command in a specific directory
fn run_gitpartial(
    cwd: &Path,
    args: &[&str],
) -> Result<String> {
    let bin_path = PathBuf::from(env!("CARGO_BIN_EXE_git-partial")); // Get binary path

    let output = Command::new(bin_path)
        .args(args)
        .current_dir(cwd) // Run in the specified directory
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

// Helper to check if a file exists and has specific content
fn file_content_matches(
    base_path: &Path,
    relative_path: &str,
    expected_content: &str,
) -> Result<bool> {
    let file_path = base_path.join(relative_path);
    if !file_path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(file_path)?;
    Ok(content == expected_content)
}

#[test]
fn test_partial_clone() -> Result<()> {
    // 1. Set up a source Git repository with test data
    let source_repo = TestRepo::new()?;
    source_repo.write_file("README.md", "# Main Readme")?;
    source_repo.write_file("src/main.rs", "fn main() {}")?;
    source_repo.write_file("src/lib.rs", "pub fn lib_func() {}")?;
    source_repo.write_file("data/data.txt", "important data")?;
    source_repo.add_all()?;
    source_repo.commit("Initial commit")?;
    let source_repo_url = source_repo.path_str()?; // Use path as URL for local clone

    // 2. Clone it partially with gitpartial
    let clone_dir = tempfile::tempdir()?;
    let clone_path = clone_dir.path();
    let workspace_dir = PathBuf::from("."); // Assuming tests run from workspace root

    run_gitpartial(
        &workspace_dir, // Run from workspace root
        &[
            "clone",
            &source_repo_url,
            &clone_path.to_string_lossy(),
            "--paths",
            "src/main.rs",
            "README.md", // Use space-separated paths
        ],
    )?;

    // 3. Verify the results
    // Check that specified files exist
    assert!(
        file_exists(clone_path, "README.md"),
        "README.md should exist"
    );
    assert!(
        file_exists(clone_path, "src/main.rs"),
        "src/main.rs should exist"
    );
    assert!(
        file_content_matches(clone_path, "README.md", "# Main Readme")?,
        "README.md content mismatch"
    );
    assert!(
        file_content_matches(clone_path, "src/main.rs", "fn main() {}")?,
        "src/main.rs content mismatch"
    );

    // Check that unspecified files do NOT exist
    assert!(
        !file_exists(clone_path, "src/lib.rs"),
        "src/lib.rs should NOT exist"
    );
    assert!(
        !file_exists(clone_path, "data/data.txt"),
        "data/data.txt should NOT exist"
    );

    // Check that metadata file exists and contains correct info (basic check)
    assert!(
        file_exists(clone_path, ".gitpartial/metadata.json"),
        ".gitpartial/metadata.json should exist"
    );
    // TODO: Add more detailed metadata content verification

    Ok(())
}

#[test]
fn test_partial_clone_with_glob_pattern() -> Result<()> {
    // 1. Set up a source Git repository
    let source_repo = TestRepo::new()?;
    source_repo.write_file("docs/README.md", "# Docs Readme")?;
    source_repo.write_file("docs/guide.md", "User guide")?;
    source_repo.write_file("src/main.rs", "fn main() {}")?;
    source_repo.write_file("images/logo.png", "binary data")?;
    source_repo.add_all()?;
    source_repo.commit("Initial commit")?;
    let source_repo_url = source_repo.path_str()?;

    // 2. Clone using a glob pattern
    let clone_dir = tempfile::tempdir()?;
    let clone_path = clone_dir.path();
    let workspace_dir = PathBuf::from(".");

    run_gitpartial(
        &workspace_dir,
        &[
            "clone",
            &source_repo_url,
            &clone_path.to_string_lossy(),
            "--paths",
            "docs/**", // Clone everything under docs/
        ],
    )?;

    // 3. Verify the results
    assert!(
        file_exists(clone_path, "docs/README.md"),
        "docs/README.md should exist"
    );
    assert!(
        file_exists(clone_path, "docs/guide.md"),
        "docs/guide.md should exist"
    );
    assert!(
        file_content_matches(clone_path, "docs/README.md", "# Docs Readme")?,
        "docs/README.md content mismatch"
    );
    assert!(
        file_content_matches(clone_path, "docs/guide.md", "User guide")?,
        "docs/guide.md content mismatch"
    );

    assert!(
        !file_exists(clone_path, "src/main.rs"),
        "src/main.rs should NOT exist"
    );
    assert!(
        !file_exists(clone_path, "images/logo.png"),
        "images/logo.png should NOT exist"
    );
    assert!(
        file_exists(clone_path, ".gitpartial/metadata.json"),
        ".gitpartial/metadata.json should exist"
    );
    // TODO: Verify metadata path contents

    Ok(())
}
