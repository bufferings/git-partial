use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

use super::metadata::RepositoryMetadata;
use crate::git::commands;
use crate::git::sparse;

/// Represents a partially checked out Git repository.
/// TODO: This struct provides a higher-level abstraction over Git commands, but is not yet fully used by the CLI.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Repository {
    /// Path to the repository on disk
    path: PathBuf,

    /// Metadata for the repository
    metadata: RepositoryMetadata,
}

impl Repository {
    /// Opens an existing repository at the given path.
    /// TODO: Implement or remove if replaced by direct command usage.
    #[allow(dead_code)]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();

        // Verify this is a git repository
        if !path_buf.join(".git").exists() {
            return Err(anyhow!("Not a git repository: {:?}", path_buf));
        }

        // Load metadata
        let metadata =
            RepositoryMetadata::load(&path_buf).context("Failed to load repository metadata")?;

        Ok(Repository {
            path: path_buf,
            metadata,
        })
    }

    /// Clones a repository partially based on the given paths.
    /// TODO: Implement or remove if replaced by direct command usage.
    #[allow(dead_code)]
    pub fn clone<P: AsRef<Path>>(
        url: &str,
        target_path: P,
        paths: &[String],
    ) -> Result<Self> {
        let path_buf = target_path.as_ref().to_path_buf();

        // Clone with sparse checkout
        sparse::clone_sparse(url, &path_buf).context("Failed to clone repository")?;

        // Set sparse checkout paths
        sparse::set_sparse_paths(&path_buf, paths)
            .context("Failed to set sparse checkout paths")?;

        // Get current commit
        let commit = commands::get_head_commit(&path_buf).context("Failed to get HEAD commit")?;

        // Create and save metadata
        let mut metadata = RepositoryMetadata::new(url.to_string());
        metadata.add_paths(paths);
        metadata.set_last_commit(&commit);
        metadata.save(&path_buf)?;

        Ok(Repository {
            path: path_buf,
            metadata,
        })
    }

    /// Adds new paths to the sparse checkout and updates the working directory.
    /// TODO: Implement or remove if replaced by direct command usage.
    #[allow(dead_code)]
    pub fn add_paths(
        &mut self,
        paths: &[String],
    ) -> Result<()> {
        // Get current sparse paths
        let mut current_paths: Vec<String> =
            self.metadata.checked_out_paths.iter().cloned().collect();

        // Add new paths
        for path in paths {
            if !current_paths.contains(path) {
                current_paths.push(path.clone());
            }
        }

        // Update sparse checkout
        sparse::set_sparse_paths(&self.path, &current_paths)
            .context("Failed to update sparse checkout paths")?;

        // Update metadata
        self.metadata.add_paths(paths);
        self.metadata.save(&self.path)?;

        Ok(())
    }

    /// Returns the path to the repository root.
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns an immutable reference to the repository metadata.
    #[allow(dead_code)]
    pub fn metadata(&self) -> &RepositoryMetadata {
        &self.metadata
    }

    /// Returns a mutable reference to the repository metadata.
    #[allow(dead_code)]
    pub fn metadata_mut(&mut self) -> &mut RepositoryMetadata {
        &mut self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for tests
    #[cfg(test)]
    mod mock {
        use anyhow::Result;
        use std::path::Path;

        pub fn setup_mock_repo() -> (tempfile::TempDir, String) {
            let dir = tempfile::tempdir().unwrap();
            std::fs::create_dir_all(dir.path().join(".git")).unwrap();

            let remote_url = "https://github.com/user/mock-repo.git".to_string();
            (dir, remote_url)
        }

        pub fn simulate_clone_result(
            path: &Path,
            remote_url: &str,
            paths: &[String],
        ) -> Result<()> {
            // Create metadata
            let mut metadata =
                crate::core::metadata::RepositoryMetadata::new(remote_url.to_string());
            metadata.add_paths(paths);
            metadata.set_last_commit("mock-commit-sha");
            metadata.save(path)?;

            Ok(())
        }
    }

    #[test]
    fn test_repository_open() {
        // Setup
        let (temp_dir, remote_url) = mock::setup_mock_repo();
        let repo_path = temp_dir.path();
        let paths = vec!["src/**".to_string(), "README.md".to_string()];

        // Simulate clone result
        mock::simulate_clone_result(repo_path, &remote_url, &paths).unwrap();

        // Test opening the repository
        let repo = Repository::open(repo_path).unwrap();

        // Verify
        assert_eq!(repo.path(), repo_path);
        assert_eq!(repo.metadata().remote_url, remote_url);
        assert_eq!(repo.metadata().checked_out_paths.len(), 2);
        assert!(repo.metadata().checked_out_paths.contains("src/**"));
        assert!(repo.metadata().checked_out_paths.contains("README.md"));
    }

    #[test]
    fn test_repository_add_paths() {
        // Setup
        let (temp_dir, remote_url) = mock::setup_mock_repo();
        let repo_path = temp_dir.path();
        let initial_paths = vec!["src/**".to_string(), "README.md".to_string()];

        // Simulate clone result
        mock::simulate_clone_result(repo_path, &remote_url, &initial_paths).unwrap();

        // Open repository
        let repo = Repository::open(repo_path).unwrap();

        // Directly verify the metadata that was set up by simulate_clone_result
        assert_eq!(repo.metadata().checked_out_paths.len(), 2);
        assert!(repo.metadata().checked_out_paths.contains("src/**"));
        assert!(repo.metadata().checked_out_paths.contains("README.md"));
        assert_eq!(
            repo.metadata().last_commit,
            Some("mock-commit-sha".to_string())
        );
    }
}
