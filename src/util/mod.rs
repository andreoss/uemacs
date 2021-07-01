pub const fn version() -> &'static str {
    "uemacs 0.1.0"
}

fn expand_tilde(prefix: &str) -> String {
    if !prefix.starts_with('~') {
        return prefix.to_string();
    }
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return prefix.to_string();
    }
    if prefix == "~" {
        return home;
    }
    if let Some(rest) = prefix.strip_prefix("~/") {
        return format!("{home}/{rest}");
    }
    prefix.to_string()
}

struct CompletionParts {
    dir_to_read: String,
    base: String,
    prefix_str: String,
}

fn resolve_completion_parts(expanded: &str) -> CompletionParts {
    use std::path::Path;
    let path = Path::new(expanded);
    let listing_dir = expanded.ends_with('/') || expanded.is_empty();
    if listing_dir {
        let d = if expanded.is_empty() {
            ".".to_string()
        } else if expanded.chars().all(|c| c == '/') {
            "/".to_string()
        } else {
            expanded.trim_end_matches('/').to_string()
        };
        let p = if expanded.is_empty() {
            String::new()
        } else {
            expanded.to_string()
        };
        CompletionParts {
            dir_to_read: if d.is_empty() { ".".to_string() } else { d },
            base: String::new(),
            prefix_str: p,
        }
    } else {
        let parent = path
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = if parent.is_empty() {
            ".".to_string()
        } else {
            parent.clone()
        };
        let base = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let prefix_str = if parent.is_empty() {
            String::new()
        } else {
            format!("{parent}/")
        };
        CompletionParts {
            dir_to_read: dir,
            base,
            prefix_str,
        }
    }
}

fn collect_completions(dir: &str, base: &str, prefix: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut matches: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if !name.starts_with(base) {
                return None;
            }
            let mut s = format!("{prefix}{name}");
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                s.push('/');
            }
            Some(s)
        })
        .collect();
    matches.sort();
    matches
}

pub fn complete_filename(prefix: &str) -> Vec<String> {
    let expanded = expand_tilde(prefix);
    let parts = resolve_completion_parts(&expanded);
    collect_completions(&parts.dir_to_read, &parts.base, &parts.prefix_str)
}

#[cfg(test)]
mod tests;
