use git_partial::core::metadata::RepositoryMetadata;
use std::collections::HashSet;

#[test]
fn test_metadata_serialization() {
    // Create a repository metadata instance
    let mut metadata = RepositoryMetadata::new("https://github.com/user/repo.git".to_string());
    metadata.add_paths(&["src/frontend/**".to_string(), "docs/*.md".to_string()]);
    metadata.set_last_commit("abcdef123456");

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&metadata).unwrap();

    // Deserialize from JSON
    let deserialized: RepositoryMetadata = serde_json::from_str(&json).unwrap();

    // Verify values
    assert_eq!(deserialized.remote_url, "https://github.com/user/repo.git");
    assert_eq!(deserialized.checked_out_paths.len(), 2);
    assert!(deserialized.checked_out_paths.contains("src/frontend/**"));
    assert!(deserialized.checked_out_paths.contains("docs/*.md"));
    assert_eq!(deserialized.last_commit, Some("abcdef123456".to_string()));
}

#[test]
fn test_metadata_merge() {
    // Create first metadata
    let mut metadata1 = RepositoryMetadata::new("https://github.com/user/repo.git".to_string());
    metadata1.add_paths(&["src/frontend/**".to_string(), "README.md".to_string()]);
    metadata1.set_last_commit("commit1");

    // Create second metadata with different paths
    let mut metadata2 = RepositoryMetadata::new("https://github.com/user/repo.git".to_string());
    metadata2.add_paths(&["src/shared/**".to_string(), "docs/**".to_string()]);
    metadata2.set_last_commit("commit2");

    // Manually merge the metadata
    let mut merged = RepositoryMetadata::new(metadata1.remote_url.clone());

    // Combine path sets
    let mut combined_paths = HashSet::new();
    combined_paths.extend(metadata1.checked_out_paths.iter().cloned());
    combined_paths.extend(metadata2.checked_out_paths.iter().cloned());

    // Convert back to Vec for add_paths
    let combined_paths_vec: Vec<String> = combined_paths.into_iter().collect();
    merged.add_paths(&combined_paths_vec);

    // Use the latest commit
    merged.set_last_commit("commit2");

    // Verify values
    assert_eq!(merged.remote_url, "https://github.com/user/repo.git");
    assert_eq!(merged.checked_out_paths.len(), 4);
    assert!(merged.checked_out_paths.contains("src/frontend/**"));
    assert!(merged.checked_out_paths.contains("README.md"));
    assert!(merged.checked_out_paths.contains("src/shared/**"));
    assert!(merged.checked_out_paths.contains("docs/**"));
    assert_eq!(merged.last_commit, Some("commit2".to_string()));
}

#[test]
fn test_metadata_empty_paths() {
    let metadata = RepositoryMetadata::new("https://github.com/user/repo.git".to_string());

    assert!(metadata.checked_out_paths.is_empty());

    // Test serialization of empty paths
    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: RepositoryMetadata = serde_json::from_str(&json).unwrap();

    assert!(deserialized.checked_out_paths.is_empty());
}
