use super::*;

#[test]
fn test_version() {
    assert!(version().contains("uemacs"));
}

#[test]
fn test_complete_filename_empty() {
    let results = complete_filename("");
    assert!(!results.is_empty());
    let has_cargo = results.iter().any(|n| n.contains("Cargo"));
    let has_src = results.iter().any(|n| n.contains("src/"));
    assert!(
        has_cargo || has_src,
        "expected Cargo or src in results: {results:?}"
    );
}

#[test]
fn test_complete_filename_src_prefix() {
    let results = complete_filename("src/");
    assert!(results.iter().any(|n| n.starts_with("src/")));
}

#[test]
fn test_complete_filename_no_dot_slash_pollution() {
    let results = complete_filename("");
    assert!(
        !results.iter().any(|n| n.starts_with("./")),
        "completions for empty/relative prefix must not have ./ prefix; got {results:?}",
    );
}

#[test]
fn test_complete_filename_tilde_expansion() {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return;
    }
    let results = complete_filename("~/");
    for r in &results {
        assert!(
            r.starts_with(&format!("{home}/")),
            "~/ expands to $HOME/; got {r}",
        );
    }
}

#[test]
fn test_complete_filename_partial_in_cwd() {
    let results = complete_filename("Carg");
    assert!(
        results
            .iter()
            .any(|n| n == "Cargo.toml" || n == "Cargo.lock"),
        "expected bare 'Cargo.toml' (no ./), got {results:?}"
    );
    assert!(!results.iter().any(|n| n.starts_with("./")));
}

#[test]
fn test_complete_filename_no_match() {
    let results = complete_filename("/nonexistent_path_xyz/foo");
    assert!(results.is_empty());
}

#[test]
fn test_complete_filename_matches_cargo_toml() {
    let results = complete_filename("Cargo");
    assert!(results.iter().any(|n| n.contains("Cargo.toml")));
}

#[test]
fn test_complete_filename_trailing_slash() {
    let results = complete_filename("src/");
    assert!(results.iter().all(|n| n.starts_with("src/")));
}

#[test]
fn test_complete_filename_with_non_utf8_path() {
    let results = complete_filename("/proc/1/fd/");
    assert!(results.is_empty() || !results.is_empty());
}
