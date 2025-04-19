use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// Represents a temporary Git repository for testing purposes.
pub struct TestRepo {
    #[allow(dead_code)] // Keep the TempDir alive for the duration of the test
    temp_dir: TempDir,
    path: PathBuf,
}

impl TestRepo {
    /// Creates a new temporary Git repository.
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let path = temp_dir.path().to_path_buf();

        // Initialize git repo
        Self::run_git_command(&path, &["init", "-b", "main"])?;
        // Set user for commits (needed for git commit)
        Self::run_git_command(&path, &["config", "user.name", "Test User"])?;
        Self::run_git_command(&path, &["config", "user.email", "test@example.com"])?;

        Ok(TestRepo { temp_dir, path })
    }

    /// Returns the path to the root of the repository.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the path to the root of the repository as a String.
    pub fn path_str(&self) -> Result<String> {
        self.path
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert repo path to string"))
            .map(|s| s.to_string())
    }

    /// Writes content to a file within the repository.
    /// Creates directories if they don't exist.
    pub fn write_file(
        &self,
        relative_path: &str,
        content: &str,
    ) -> Result<()> {
        let file_path = self.path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file_path, content)?;
        Ok(())
    }

    /// Runs `git add .` in the repository.
    pub fn add_all(&self) -> Result<()> {
        Self::run_git_command(self.path(), &["add", "."])?;
        Ok(())
    }

    /// Runs `git commit -m <message>` in the repository.
    /// Returns the SHA of the new commit.
    pub fn commit(
        &self,
        message: &str,
    ) -> Result<String> {
        Self::run_git_command(self.path(), &["commit", "-m", message])?;
        // After commit, get the new HEAD SHA
        let output = Self::run_git_command(self.path(), &["rev-parse", "HEAD"])?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string()) // Return the SHA
    }

    /// Helper function to run a Git command within the repository.
    pub fn run_git_command(
        repo_path: &Path,
        args: &[&str],
    ) -> Result<Output> {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow!(
                "Git command failed in {}:
Args: git {:?}
Exit Code: {:?}
Stderr: {}
Stdout: {}",
                repo_path.display(),
                args,
                output.status.code(),
                stderr,
                stdout
            ));
        }
        Ok(output)
    }
}

/// Creates a temporary clone directory for testing clone operations
pub fn create_clone_dir() -> (TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary clone directory");
    let clone_path = temp_dir.path().to_path_buf();
    (temp_dir, clone_path)
}

/// Verifies if a file exists in the repository
pub fn file_exists(
    repo_path: &Path,
    file_path: &str,
) -> bool {
    repo_path.join(file_path).exists()
}

/// Gets a list of files in a directory (recursively)
pub fn list_files(dir_path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if dir_path.exists() && dir_path.is_dir() {
        for entry in walkdir::WalkDir::new(dir_path)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
    }

    files
}
