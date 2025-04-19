use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Metadata for a GitPartial repository
#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    /// The original repository URL
    pub remote_url: String,

    /// The set of paths that have been checked out
    pub checked_out_paths: HashSet<String>,

    /// The last known commit SHA
    pub last_commit: Option<String>,
}

impl RepositoryMetadata {
    /// Creates a new metadata instance for a repository
    pub fn new(remote_url: String) -> Self {
        RepositoryMetadata {
            remote_url,
            checked_out_paths: HashSet::new(),
            last_commit: None,
        }
    }

    /// Adds paths to the checked out paths set
    pub fn add_paths(
        &mut self,
        paths: &[String],
    ) {
        for path in paths {
            self.checked_out_paths.insert(path.clone());
        }
    }

    /// Sets the last commit SHA
    pub fn set_last_commit(
        &mut self,
        commit_sha: &str,
    ) {
        self.last_commit = Some(commit_sha.to_string());
    }

    /// Saves metadata to the specified repository path
    pub fn save<P: AsRef<Path>>(
        &self,
        repo_path: P,
    ) -> Result<()> {
        let metadata_path = Self::metadata_path(&repo_path);

        // Create gitpartial directory if it doesn't exist
        let gitpartial_dir = metadata_path.parent().unwrap();
        fs::create_dir_all(gitpartial_dir)
            .with_context(|| format!("Failed to create directory: {:?}", gitpartial_dir))?;

        let serialized =
            serde_json::to_string_pretty(self).context("Failed to serialize metadata")?;

        fs::write(&metadata_path, serialized)
            .with_context(|| format!("Failed to write metadata to {:?}", metadata_path))?;

        Ok(())
    }

    /// Loads metadata from the specified repository path
    pub fn load<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let metadata_path = Self::metadata_path(&repo_path);

        let content = fs::read_to_string(&metadata_path)
            .with_context(|| format!("Failed to read metadata from {:?}", metadata_path))?;

        let metadata = serde_json::from_str(&content).context("Failed to deserialize metadata")?;

        Ok(metadata)
    }

    /// Returns the path to the metadata file
    fn metadata_path<P: AsRef<Path>>(repo_path: P) -> PathBuf {
        repo_path.as_ref().join(".gitpartial").join("metadata.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory")
    }

    #[test]
    fn test_metadata_creation() {
        let metadata = RepositoryMetadata::new("https://github.com/user/repo.git".to_string());

        assert_eq!(metadata.remote_url, "https://github.com/user/repo.git");
        assert!(metadata.checked_out_paths.is_empty());
        assert_eq!(metadata.last_commit, None);
    }

    #[test]
    fn test_add_paths() {
        let mut metadata = RepositoryMetadata::new("https://github.com/user/repo.git".to_string());

        metadata.add_paths(&["src/frontend/**".to_string(), "*.md".to_string()]);

        assert_eq!(metadata.checked_out_paths.len(), 2);
        assert!(metadata.checked_out_paths.contains("src/frontend/**"));
        assert!(metadata.checked_out_paths.contains("*.md"));
    }

    #[test]
    fn test_set_last_commit() {
        let mut metadata = RepositoryMetadata::new("https://github.com/user/repo.git".to_string());

        metadata.set_last_commit("abc123");

        assert_eq!(metadata.last_commit, Some("abc123".to_string()));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create .git directory
        fs::create_dir_all(repo_path.join(".git")).expect("Failed to create .git directory");

        let mut metadata = RepositoryMetadata::new("https://github.com/user/repo.git".to_string());
        metadata.add_paths(&["src/**".to_string(), "README.md".to_string()]);
        metadata.set_last_commit("def456");

        // Save metadata
        metadata.save(repo_path).expect("Failed to save metadata");

        // Check that file exists
        let metadata_path = RepositoryMetadata::metadata_path(repo_path);
        assert!(metadata_path.exists());

        // Load metadata
        let loaded = RepositoryMetadata::load(repo_path).expect("Failed to load metadata");

        // Verify loaded data
        assert_eq!(loaded.remote_url, "https://github.com/user/repo.git");
        assert_eq!(loaded.checked_out_paths.len(), 2);
        assert!(loaded.checked_out_paths.contains("src/**"));
        assert!(loaded.checked_out_paths.contains("README.md"));
        assert_eq!(loaded.last_commit, Some("def456".to_string()));
    }
}
