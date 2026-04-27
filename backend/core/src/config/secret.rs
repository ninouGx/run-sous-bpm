/// Reads a secret value from `{var}_FILE` (file path) if set, otherwise from `{var}` directly.
///
/// # Panics
///
/// Panics if neither `{var}` nor `{var}_FILE` is set, or if the file cannot be read.
#[must_use]
pub fn read_secret(var: &str) -> String {
    if let Ok(path) = std::env::var(format!("{var}_FILE")) {
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {var}_FILE at {path}: {e}"))
            .trim_end()
            .to_string()
    } else {
        std::env::var(var).unwrap_or_else(|_| panic!("Either {var} or {var}_FILE must be set"))
    }
}
