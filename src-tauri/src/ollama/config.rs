use std::sync::{LazyLock, RwLock};

const DEFAULT_HOST: &str = "http://127.0.0.1:11434";

static OLLAMA_HOST: LazyLock<RwLock<String>> =
    LazyLock::new(|| RwLock::new(initial_host()));

fn initial_host() -> String {
    std::env::var("OLLAMA_HOST")
        .ok()
        .and_then(|raw| normalize_host(&raw).ok())
        .unwrap_or_else(|| DEFAULT_HOST.to_string())
}

pub fn get_host() -> String {
    OLLAMA_HOST
        .read()
        .expect("ollama host lock poisoned")
        .clone()
}

pub fn set_host(input: &str) -> Result<String, String> {
    let normalized = normalize_host(input)?;
    *OLLAMA_HOST
        .write()
        .map_err(|_| "ollama host lock poisoned".to_string())? = normalized.clone();
    Ok(normalized)
}

pub fn normalize_host(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Ollama URL cannot be empty".into());
    }

    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };

    let without_trailing_slash = with_scheme.trim_end_matches('/').to_string();
    if without_trailing_slash.len() <= "http://".len() {
        return Err("Ollama URL is invalid".into());
    }

    Ok(without_trailing_slash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_scheme_and_trims_slash() {
        assert_eq!(
            normalize_host("127.0.0.1:11434/").unwrap(),
            "http://127.0.0.1:11434"
        );
    }
}
