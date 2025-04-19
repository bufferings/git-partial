use glob::Pattern;
use std::path::Path;

/// Represents a set of glob patterns for selecting paths.
/// TODO: This struct and its methods are not yet integrated into the main commands.
#[allow(dead_code)]
#[derive(Debug)]
pub struct PathSelector {
    patterns: Vec<Pattern>,
}

impl PathSelector {
    /// Creates a new PathSelector with the given glob patterns
    #[allow(dead_code)] // TODO: Not yet integrated
    pub fn new(patterns: Vec<&str>) -> Self {
        let compiled_patterns = patterns
            .into_iter()
            .map(|p| Pattern::new(p).expect("Invalid glob pattern"))
            .collect();

        PathSelector {
            patterns: compiled_patterns,
        }
    }

    /// Checks if a given path matches any of the patterns
    #[allow(dead_code)] // TODO: Not yet integrated
    pub fn matches<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> bool {
        let path_str = path.as_ref().to_string_lossy();

        self.patterns
            .iter()
            .any(|pattern| pattern.matches(&path_str))
    }

    /// Returns the underlying glob patterns.
    #[allow(dead_code)] // TODO: Not yet integrated
    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_matching_basic() {
        let selector = PathSelector::new(vec!["src/frontend/**", "*.md"]);

        assert!(selector.matches("src/frontend/components/Button.js"));
        assert!(selector.matches("README.md"));
        assert!(!selector.matches("src/backend/server.js"));
    }

    #[test]
    fn test_path_matching_empty() {
        let selector = PathSelector::new(vec![]);

        assert!(!selector.matches("any/path.txt"));
    }

    #[test]
    fn test_path_matching_exact() {
        let selector = PathSelector::new(vec!["exact.txt"]);

        assert!(selector.matches("exact.txt"));
        assert!(!selector.matches("not_exact.txt"));
        assert!(!selector.matches("path/to/exact.txt"));
    }

    #[test]
    fn test_path_matching_complex() {
        // Use simpler pattern matching for tests
        let selector = PathSelector::new(vec![
            "src/frontend/**/*.js",
            "src/shared/**/*.js",
            "src/frontend/**/*.jsx",
            "docs/**/*.md",
        ]);

        assert!(selector.matches("src/frontend/components/Button.js"));
        assert!(selector.matches("src/shared/utils/format.js"));
        assert!(selector.matches("src/frontend/pages/Home.jsx"));
        assert!(selector.matches("docs/api/v1/endpoints.md"));

        assert!(!selector.matches("src/backend/api/routes.js"));
        assert!(!selector.matches("src/frontend/styles.css"));
        assert!(!selector.matches("README.md"));
    }
}
