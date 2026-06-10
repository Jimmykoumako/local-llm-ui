use super::permissions::FsAuthConfig;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use walkdir::WalkDir;

pub fn read_file(path: &Path, offset: usize, limit: usize, config: &FsAuthConfig) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| format!("stat failed: {e}"))?;
    if meta.len() > config.max_read_bytes && limit == 0 {
        return Err(format!(
            "File too large ({} bytes). Max {} bytes. Use offset/limit.",
            meta.len(),
            config.max_read_bytes
        ));
    }

    let content = fs::read_to_string(path).map_err(|e| format!("read failed: {e}"))?;
    let lines: Vec<&str> = content.lines().collect();

    if offset > 0 || limit > 0 {
        let start = offset.saturating_sub(1);
        let end = if limit > 0 {
            (start + limit).min(lines.len())
        } else {
            lines.len()
        };
        let slice = &lines[start..end];
        Ok(json!({
            "path": path.display().to_string(),
            "total_lines": lines.len(),
            "offset": offset,
            "limit": limit,
            "content": slice.join("\n"),
        })
        .to_string())
    } else if content.len() as u64 > config.max_read_bytes {
        let truncated: String = content.chars().take(config.max_read_bytes as usize).collect();
        Ok(json!({
            "path": path.display().to_string(),
            "truncated": true,
            "content": truncated,
        })
        .to_string())
    } else {
        Ok(json!({
            "path": path.display().to_string(),
            "content": content,
        })
        .to_string())
    }
}

pub fn write_file(path: &Path, content: &str) -> Result<String, String> {
    let existed = path.exists();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
    }
    fs::write(path, content).map_err(|e| format!("write failed: {e}"))?;
    Ok(json!({
        "path": path.display().to_string(),
        "bytes_written": content.len(),
        "created": !existed,
        "overwritten": existed,
    })
    .to_string())
}

pub fn create_directory(path: &Path) -> Result<String, String> {
    fs::create_dir_all(path).map_err(|e| format!("mkdir failed: {e}"))?;
    Ok(json!({ "path": path.display().to_string(), "created": true }).to_string())
}

pub fn list_directory(path: &Path, recursive: bool) -> Result<String, String> {
    let mut entries: Vec<Value> = Vec::new();

    if recursive {
        for entry in WalkDir::new(path)
            .max_depth(6)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            let meta = entry.metadata().ok();
            entries.push(json!({
                "path": p.display().to_string(),
                "name": p.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                "is_dir": meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                "size": meta.as_ref().map(|m| m.len()).unwrap_or(0),
            }));
        }
    } else {
        for entry in fs::read_dir(path).map_err(|e| format!("list failed: {e}"))? {
            let entry = entry.map_err(|e| format!("list entry failed: {e}"))?;
            let meta = entry.metadata().map_err(|e| format!("stat failed: {e}"))?;
            let p = entry.path();
            entries.push(json!({
                "path": p.display().to_string(),
                "name": p.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                "is_dir": meta.is_dir(),
                "size": meta.len(),
            }));
        }
    }

    Ok(json!({
        "path": path.display().to_string(),
        "recursive": recursive,
        "count": entries.len(),
        "entries": entries,
    })
    .to_string())
}

pub fn delete_path(path: &Path) -> Result<String, String> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| format!("delete dir failed: {e}"))?;
    } else {
        fs::remove_file(path).map_err(|e| format!("delete file failed: {e}"))?;
    }
    Ok(json!({ "path": path.display().to_string(), "deleted": true }).to_string())
}

pub fn move_path(from: &Path, to: &Path) -> Result<String, String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
    }
    fs::rename(from, to).map_err(|e| format!("move failed: {e}"))?;
    Ok(json!({
        "from": from.display().to_string(),
        "to": to.display().to_string(),
        "moved": true,
    })
    .to_string())
}

pub fn copy_path(from: &Path, to: &Path) -> Result<String, String> {
    if from.is_dir() {
        copy_dir_recursive(from, to)?;
    } else {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
        }
        fs::copy(from, to).map_err(|e| format!("copy failed: {e}"))?;
    }
    Ok(json!({
        "from": from.display().to_string(),
        "to": to.display().to_string(),
        "copied": true,
    })
    .to_string())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("mkdir failed: {e}"))?;
    for entry in fs::read_dir(from).map_err(|e| format!("read dir failed: {e}"))? {
        let entry = entry.map_err(|e| format!("read entry failed: {e}"))?;
        let src = entry.path();
        let dest = to.join(entry.file_name());
        if src.is_dir() {
            copy_dir_recursive(&src, &dest)?;
        } else {
            fs::copy(&src, &dest).map_err(|e| format!("copy failed: {e}"))?;
        }
    }
    Ok(())
}

pub fn stat_file(path: &Path) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| format!("stat failed: {e}"))?;
    Ok(json!({
        "path": path.display().to_string(),
        "is_dir": meta.is_dir(),
        "is_file": meta.is_file(),
        "size": meta.len(),
        "readonly": meta.permissions().readonly(),
    })
    .to_string())
}

pub fn search_files(root: &Path, pattern: &str, config: &FsAuthConfig) -> Result<String, String> {
    let pattern_lower = pattern.to_lowercase();
    let mut matches: Vec<Value> = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if matches.len() >= config.max_search_results {
            break;
        }
        let p = entry.path();
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if glob_match(&pattern_lower, &name) || name.contains(&pattern_lower) {
            matches.push(json!({
                "path": p.display().to_string(),
                "name": p.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                "is_dir": entry.file_type().is_dir(),
            }));
        }
    }

    Ok(json!({
        "root": root.display().to_string(),
        "pattern": pattern,
        "count": matches.len(),
        "matches": matches,
    })
    .to_string())
}

pub fn search_content(
    root: &Path,
    query: &str,
    file_glob: Option<&str>,
    config: &FsAuthConfig,
) -> Result<String, String> {
    let query_lower = query.to_lowercase();
    let glob_filter = file_glob.map(|g| g.to_lowercase());
    let mut hits: Vec<Value> = Vec::new();

    'walk: for entry in WalkDir::new(root)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if hits.len() >= config.max_search_results {
            break;
        }
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if let Some(ref g) = glob_filter {
            if !glob_match(g, &name) {
                continue;
            }
        }
        if let Ok(meta) = p.metadata() {
            if meta.len() > config.max_read_bytes {
                continue;
            }
        }
        if !is_probably_text(p) {
            continue;
        }

        let file = match fs::File::open(p) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let reader = BufReader::new(file);
        for (line_no, line) in reader.lines().enumerate() {
            if hits.len() >= config.max_search_results {
                break 'walk;
            }
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            if line.to_lowercase().contains(&query_lower) {
                hits.push(json!({
                    "path": p.display().to_string(),
                    "line": line_no + 1,
                    "text": line,
                }));
            }
        }
    }

    Ok(json!({
        "root": root.display().to_string(),
        "query": query,
        "count": hits.len(),
        "hits": hits,
    })
    .to_string())
}

fn is_probably_text(path: &Path) -> bool {
    let Ok(mut f) = fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 512];
    let n = f.read(&mut buf).unwrap_or(0);
    if n == 0 {
        return true;
    }
    !buf[..n].contains(&0)
}

fn glob_match(pattern: &str, name: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let (pre, suf) = (parts[0], parts[1]);
            return name.starts_with(pre) && name.ends_with(suf);
        }
    }
    pattern == name
}
