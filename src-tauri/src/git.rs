use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

const MAX_OUTPUT_BYTES: usize = 256_000;

pub fn git_status(cwd: &Path) -> Result<String, String> {
    run_git(&["status", "--porcelain", "-b"], cwd, |o| {
        json!({
            "cwd": cwd.display().to_string(),
            "output": truncate(&o.stdout),
        })
    })
}

pub fn git_diff(cwd: &Path, staged: bool, path: Option<&str>) -> Result<String, String> {
    let mut args = vec!["diff"];
    if staged {
        args.push("--staged");
    }
    if let Some(p) = path {
        args.push(p);
    }
    run_git(&args, cwd, |o| {
        json!({
            "cwd": cwd.display().to_string(),
            "staged": staged,
            "diff": truncate(&o.stdout),
        })
    })
}

pub fn git_log(cwd: &Path, limit: usize) -> Result<String, String> {
    let n = limit.min(50).max(1);
    let limit_str = n.to_string();
    run_git(
        &["log", "--oneline", "-n", &limit_str],
        cwd,
        |o| {
            json!({
                "cwd": cwd.display().to_string(),
                "log": truncate(&o.stdout),
            })
        },
    )
}

pub fn git_branch_list(cwd: &Path) -> Result<String, String> {
    run_git(&["branch", "-a", "-v"], cwd, |o| {
        json!({
            "cwd": cwd.display().to_string(),
            "branches": truncate(&o.stdout),
        })
    })
}

pub fn git_add(cwd: &Path, paths: &[String]) -> Result<String, String> {
    let mut args = vec!["add"];
    for p in paths {
        args.push(p.as_str());
    }
    run_git(&args, cwd, |o| {
        json!({
            "cwd": cwd.display().to_string(),
            "paths": paths,
            "output": truncate(&o.stdout),
        })
    })
}

pub fn git_commit(cwd: &Path, message: &str) -> Result<String, String> {
    run_git(&["commit", "-m", message], cwd, |o| {
        json!({
            "cwd": cwd.display().to_string(),
            "message": message,
            "output": truncate(&o.stdout),
            "stderr": truncate(&o.stderr),
        })
    })
}

pub fn git_checkout(cwd: &Path, branch: &str) -> Result<String, String> {
    run_git(&["checkout", branch], cwd, |o| {
        json!({
            "cwd": cwd.display().to_string(),
            "branch": branch,
            "output": truncate(&o.stdout),
            "stderr": truncate(&o.stderr),
        })
    })
}

pub fn git_pull(cwd: &Path, remote: Option<&str>, branch: Option<&str>) -> Result<String, String> {
    let mut args = vec!["pull"];
    if let Some(r) = remote {
        args.push(r);
    }
    if let Some(b) = branch {
        args.push(b);
    }
    run_git(&args, cwd, |o| {
        json!({
            "cwd": cwd.display().to_string(),
            "output": truncate(&o.stdout),
            "stderr": truncate(&o.stderr),
        })
    })
}

pub fn git_push(cwd: &Path, remote: Option<&str>, branch: Option<&str>) -> Result<String, String> {
    let mut args = vec!["push"];
    if let Some(r) = remote {
        args.push(r);
    }
    if let Some(b) = branch {
        args.push(b);
    }
    run_git(&args, cwd, |o| {
        json!({
            "cwd": cwd.display().to_string(),
            "output": truncate(&o.stdout),
            "stderr": truncate(&o.stderr),
        })
    })
}

struct GitOutput {
    stdout: String,
    stderr: String,
}

fn run_git<F>(args: &[&str], cwd: &Path, map: F) -> Result<String, String>
where
    F: FnOnce(GitOutput) -> Value,
{
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    let result = map(GitOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    });

    if output.status.success() {
        Ok(result.to_string())
    } else {
        Err(json!({
            "error": "git command failed",
            "args": args,
            "exit_code": output.status.code(),
            "stderr": truncate(&String::from_utf8_lossy(&output.stderr)),
            "stdout": truncate(&String::from_utf8_lossy(&output.stdout)),
        })
        .to_string())
    }
}

fn truncate(text: &str) -> String {
    if text.len() > MAX_OUTPUT_BYTES {
        let mut s = text[..MAX_OUTPUT_BYTES].to_string();
        s.push_str("\n…[output truncated]");
        s
    } else {
        text.to_string()
    }
}

pub fn paths_from_args(args: &Value) -> Result<Vec<String>, String> {
    if let Some(arr) = args.get("paths").and_then(|v| v.as_array()) {
        return Ok(arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect());
    }
    if let Some(s) = args.get("path").and_then(|v| v.as_str()) {
        return Ok(vec![s.to_string()]);
    }
    Err("Missing paths (array) or path (string)".into())
}
