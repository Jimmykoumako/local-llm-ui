use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsAuthConfig {
    pub level: String,
    pub allowed_roots: Vec<String>,
    #[serde(default = "default_max_read")]
    pub max_read_bytes: u64,
    #[serde(default = "default_max_search")]
    pub max_search_results: usize,
    #[serde(default)]
    pub shell_enabled: bool,
    #[serde(default)]
    pub git_enabled: bool,
}

fn default_max_read() -> u64 {
    512_000
}

fn default_max_search() -> usize {
    50
}

fn resolve_absolute(path: &str) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        Ok(candidate)
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Cannot resolve working directory: {e}"))
            .map(|cwd| cwd.join(candidate))
    }
}

fn is_within_roots(resolved: &Path, roots: &[String]) -> bool {
    for root in roots {
        let root_path = PathBuf::from(root);
        let root_canonical = root_path.canonicalize().unwrap_or(root_path);
        if resolved.starts_with(&root_canonical) {
            return true;
        }
    }
    false
}

pub fn canonicalize_within_roots(path: &str, roots: &[String]) -> Result<PathBuf, String> {
    if roots.is_empty() {
        return Err("No allowed workspace roots configured".into());
    }

    let absolute = resolve_absolute(path)?;
    let canonical = absolute
        .canonicalize()
        .map_err(|e| format!("Path not found or inaccessible: {path} ({e})"))?;

    if is_within_roots(&canonical, roots) {
        Ok(canonical)
    } else {
        Err(format!(
            "Path is outside allowed workspace roots: {}",
            canonical.display()
        ))
    }
}

/// Resolve paths that may not exist yet (writes, mkdir). Canonicalizes the nearest existing ancestor.
pub fn resolve_within_roots(path: &str, roots: &[String]) -> Result<PathBuf, String> {
    if roots.is_empty() {
        return Err("No allowed workspace roots configured".into());
    }

    let absolute = resolve_absolute(path)?;
    if let Ok(canonical) = absolute.canonicalize() {
        if is_within_roots(&canonical, roots) {
            return Ok(canonical);
        }
        return Err(format!(
            "Path is outside allowed workspace roots: {}",
            canonical.display()
        ));
    }

    let mut probe = absolute.as_path();
    while !probe.exists() {
        match probe.parent() {
            Some(parent) => probe = parent,
            None => {
                return Err(format!("Cannot resolve path within workspace: {path}"));
            }
        }
    }

    let base = probe
        .canonicalize()
        .map_err(|e| format!("Cannot resolve parent path: {e}"))?;
    if !is_within_roots(&base, roots) {
        return Err(format!(
            "Path is outside allowed workspace roots: {}",
            absolute.display()
        ));
    }

    let suffix = absolute
        .strip_prefix(probe)
        .unwrap_or(absolute.as_path());
    Ok(base.join(suffix))
}

pub fn resolve_cwd(cwd: Option<&str>, roots: &[String]) -> Result<PathBuf, String> {
    match cwd {
        Some(path) if !path.is_empty() => canonicalize_within_roots(path, roots),
        _ => {
            let cwd = std::env::current_dir()
                .map_err(|e| format!("Cannot resolve working directory: {e}"))?;
            let canonical = cwd
                .canonicalize()
                .map_err(|e| format!("Cannot canonicalize cwd: {e}"))?;
            if is_within_roots(&canonical, roots) {
                Ok(canonical)
            } else if let Some(first) = roots.first() {
                canonicalize_within_roots(first, roots)
            } else {
                Err("No allowed workspace roots configured".into())
            }
        }
    }
}

#[allow(dead_code)]
pub fn is_safe_relative_segment(path: &Path) -> bool {
    !path.components().any(|c| matches!(c, Component::ParentDir))
}

pub fn is_git_read_tool(tool: &str) -> bool {
    matches!(
        tool,
        "git_status" | "git_diff" | "git_log" | "git_branch_list"
    )
}

pub fn is_git_write_tool(tool: &str) -> bool {
    matches!(
        tool,
        "git_add" | "git_commit" | "git_checkout" | "git_pull" | "git_push"
    )
}

pub fn allowed_for_level(tool: &str, level: &str, shell_enabled: bool, git_enabled: bool) -> bool {
    if level == "off" {
        return false;
    }

    if tool == "run_terminal" {
        return level == "full" && shell_enabled;
    }

    if is_git_read_tool(tool) {
        return git_enabled && level != "off";
    }

    if is_git_write_tool(tool) {
        if !git_enabled {
            return false;
        }
        return match tool {
            "git_add" => matches!(level, "write" | "full"),
            _ => level == "full",
        };
    }

    match level {
        "read" => matches!(
            tool,
            "read_file" | "list_directory" | "search_files" | "search_content" | "stat_file"
        ),
        "write" => matches!(
            tool,
            "read_file"
                | "list_directory"
                | "search_files"
                | "search_content"
                | "stat_file"
                | "write_file"
                | "create_file"
                | "create_directory"
                | "move_path"
                | "copy_path"
        ),
        "full" => !tool.starts_with("git_") && tool != "run_terminal",
        _ => false,
    }
}

pub fn is_write_operation(tool: &str) -> bool {
    matches!(
        tool,
        "write_file"
            | "create_file"
            | "create_directory"
            | "move_path"
            | "copy_path"
            | "delete_path"
            | "run_terminal"
            | "git_add"
            | "git_commit"
            | "git_checkout"
            | "git_pull"
            | "git_push"
    )
}

pub fn is_destructive_operation(tool: &str) -> bool {
    matches!(
        tool,
        "delete_path"
            | "write_file"
            | "move_path"
            | "run_terminal"
            | "git_commit"
            | "git_checkout"
            | "git_pull"
            | "git_push"
    )
}

pub fn always_requires_approval(tool: &str) -> bool {
    tool == "run_terminal"
}
