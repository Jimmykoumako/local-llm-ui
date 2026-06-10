mod ops;
mod permissions;

use crate::platform;

use permissions::{
    allowed_for_level, canonicalize_within_roots, is_destructive_operation, is_write_operation,
    resolve_cwd, resolve_within_roots, FsAuthConfig,
};
use serde_json::Value;

#[tauri::command]
pub fn agent_get_default_roots() -> Vec<String> {
    let mut roots = Vec::new();
    if let Some(home) = platform::home_dir() {
        roots.push(home.clone());
        if let Some(docs) = platform::documents_dir() {
            roots.push(docs);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(c) = cwd.canonicalize() {
            roots.push(c.display().to_string());
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

#[tauri::command]
pub fn agent_execute_tool(
    name: String,
    args: Value,
    config: FsAuthConfig,
) -> Result<String, String> {
    if config.level == "off" {
        return Err("Agent file access is disabled".into());
    }
    if !allowed_for_level(
        &name,
        &config.level,
        config.shell_enabled,
        config.git_enabled,
    ) {
        return Err(format!(
            "Operation '{name}' is not allowed at permission level '{}'. \
             Raise access in Settings → Agent. Do not retry this operation.",
            config.level
        ));
    }

    match name.as_str() {
        "read_file" => {
            let path = arg_str(&args, "path")?;
            let resolved = canonicalize_within_roots(&path, &config.allowed_roots)?;
            let offset = arg_usize(&args, "offset").unwrap_or(0);
            let limit = arg_usize(&args, "limit").unwrap_or(0);
            ops::read_file(&resolved, offset, limit, &config)
        }
        "write_file" => {
            let path = arg_str(&args, "path")?;
            let content = arg_str(&args, "content")?;
            let resolved = resolve_within_roots(&path, &config.allowed_roots)?;
            ops::write_file(&resolved, &content)
        }
        "create_file" => {
            let path = arg_str(&args, "path")?;
            let content = arg_str(&args, "content").unwrap_or_default();
            let resolved = resolve_within_roots(&path, &config.allowed_roots)?;
            if resolved.exists() {
                return Err(format!("File already exists: {}", resolved.display()));
            }
            ops::write_file(&resolved, &content)
        }
        "create_directory" => {
            let path = arg_str(&args, "path")?;
            let resolved = resolve_within_roots(&path, &config.allowed_roots)?;
            ops::create_directory(&resolved)
        }
        "list_directory" => {
            let path = arg_str(&args, "path")?;
            let recursive = arg_bool(&args, "recursive").unwrap_or(false);
            let resolved = canonicalize_within_roots(&path, &config.allowed_roots)?;
            ops::list_directory(&resolved, recursive)
        }
        "delete_path" => {
            let path = arg_str(&args, "path")?;
            let resolved = canonicalize_within_roots(&path, &config.allowed_roots)?;
            ops::delete_path(&resolved)
        }
        "move_path" => {
            let from = arg_str(&args, "from")?;
            let to = arg_str(&args, "to")?;
            let from_r = canonicalize_within_roots(&from, &config.allowed_roots)?;
            let to_r = resolve_within_roots(&to, &config.allowed_roots)?;
            ops::move_path(&from_r, &to_r)
        }
        "copy_path" => {
            let from = arg_str(&args, "from")?;
            let to = arg_str(&args, "to")?;
            let from_r = canonicalize_within_roots(&from, &config.allowed_roots)?;
            let to_r = resolve_within_roots(&to, &config.allowed_roots)?;
            ops::copy_path(&from_r, &to_r)
        }
        "stat_file" => {
            let path = arg_str(&args, "path")?;
            let resolved = canonicalize_within_roots(&path, &config.allowed_roots)?;
            ops::stat_file(&resolved)
        }
        "search_files" => {
            let root = arg_str(&args, "root").or_else(|_| arg_str(&args, "path"))?;
            let pattern = arg_str(&args, "pattern")?;
            let resolved = canonicalize_within_roots(&root, &config.allowed_roots)?;
            ops::search_files(&resolved, &pattern, &config)
        }
        "search_content" => {
            let root = arg_str(&args, "root").or_else(|_| arg_str(&args, "path"))?;
            let query = arg_str(&args, "query")?;
            let file_glob = args
                .get("glob")
                .or_else(|| args.get("file_glob"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let resolved = canonicalize_within_roots(&root, &config.allowed_roots)?;
            ops::search_content(&resolved, &query, file_glob.as_deref(), &config)
        }
        "run_terminal" => {
            let command = arg_str(&args, "command")?;
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            let timeout = args
                .get("timeout_secs")
                .and_then(|v| v.as_u64());
            crate::shell::run_terminal(&command, &cwd, timeout)
        }
        "git_status" => {
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            crate::git::git_status(&cwd)
        }
        "git_diff" => {
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            let staged = arg_bool(&args, "staged").unwrap_or(false);
            let path = args.get("path").and_then(|v| v.as_str());
            crate::git::git_diff(&cwd, staged, path)
        }
        "git_log" => {
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            let limit = arg_usize(&args, "limit").unwrap_or(20);
            crate::git::git_log(&cwd, limit)
        }
        "git_branch_list" => {
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            crate::git::git_branch_list(&cwd)
        }
        "git_add" => {
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            let paths = crate::git::paths_from_args(&args)?;
            crate::git::git_add(&cwd, &paths)
        }
        "git_commit" => {
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            let message = arg_str(&args, "message")?;
            crate::git::git_commit(&cwd, &message)
        }
        "git_checkout" => {
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            let branch = arg_str(&args, "branch")?;
            crate::git::git_checkout(&cwd, &branch)
        }
        "git_pull" => {
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            let remote = args.get("remote").and_then(|v| v.as_str());
            let branch = args.get("branch").and_then(|v| v.as_str());
            crate::git::git_pull(&cwd, remote, branch)
        }
        "git_push" => {
            let cwd = resolve_cwd(args.get("cwd").and_then(|v| v.as_str()), &config.allowed_roots)?;
            let remote = args.get("remote").and_then(|v| v.as_str());
            let branch = args.get("branch").and_then(|v| v.as_str());
            crate::git::git_push(&cwd, remote, branch)
        }
        _ => Err(format!("Unknown agent tool: {name}")),
    }
}

/// Classify operations for frontend approval UI (no side effects).
#[tauri::command]
pub fn agent_classify_tool(name: String) -> Value {
    serde_json::json!({
        "write": is_write_operation(&name),
        "destructive": is_destructive_operation(&name),
        "alwaysAsk": permissions::always_requires_approval(&name),
    })
}

fn arg_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing required argument: {key}"))
}

fn arg_usize(args: &Value, key: &str) -> Option<usize> {
    args.get(key).and_then(|v| {
        v.as_u64()
            .map(|n| n as usize)
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    })
}

fn arg_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(|v| v.as_bool())
}
