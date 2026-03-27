pub fn normalize_path(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim()
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_string()
}

pub fn matches_paths(prefixes: &[String], changed_paths: &[String]) -> bool {
    if prefixes.is_empty() {
        return true;
    }
    if changed_paths.is_empty() {
        return false;
    }

    let normalized_prefixes = prefixes
        .iter()
        .map(|prefix| normalize_path(prefix))
        .filter(|prefix| !prefix.is_empty())
        .collect::<Vec<_>>();
    let normalized_changed = changed_paths
        .iter()
        .map(|changed| normalize_path(changed))
        .filter(|changed| !changed.is_empty())
        .collect::<Vec<_>>();

    normalized_changed.iter().any(|changed| {
        normalized_prefixes
            .iter()
            .any(|prefix| changed == prefix || changed.starts_with(&format!("{}/", prefix)))
    })
}
