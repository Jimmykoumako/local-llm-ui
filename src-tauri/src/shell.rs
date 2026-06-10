use serde_json::json;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const MAX_TIMEOUT_SECS: u64 = 120;
const MAX_OUTPUT_BYTES: usize = 256_000;

pub fn run_terminal(
    command: &str,
    cwd: &Path,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    if command.trim().is_empty() {
        return Err("Command cannot be empty".into());
    }

    let timeout = timeout_secs
        .unwrap_or(DEFAULT_TIMEOUT_SECS)
        .min(MAX_TIMEOUT_SECS);

    let command_owned = command.to_string();
    let cwd_owned = cwd.to_path_buf();
    let start = Instant::now();

    let handle = thread::spawn(move || run_shell_command(&command_owned, &cwd_owned));

    while !handle.is_finished() {
        if start.elapsed() > Duration::from_secs(timeout) {
            return Err(format!("Command timed out after {timeout}s"));
        }
        thread::sleep(Duration::from_millis(50));
    }

    let output = handle
        .join()
        .map_err(|_| "Shell thread panicked".to_string())?
        .map_err(|e| format!("Failed to run command: {e}"))?;

    Ok(json!({
        "command": command,
        "cwd": cwd.display().to_string(),
        "exit_code": output.status.code(),
        "stdout": truncate_output(String::from_utf8_lossy(&output.stdout).into_owned()),
        "stderr": truncate_output(String::from_utf8_lossy(&output.stderr).into_owned()),
        "duration_ms": start.elapsed().as_millis(),
    })
    .to_string())
}

fn run_shell_command(command: &str, cwd: &Path) -> std::io::Result<std::process::Output> {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", command])
            .current_dir(cwd)
            .output()
    }

    #[cfg(not(windows))]
    {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(cwd)
            .output()
    }
}

fn truncate_output(mut text: String) -> String {
    if text.len() > MAX_OUTPUT_BYTES {
        text.truncate(MAX_OUTPUT_BYTES);
        text.push_str("\n…[output truncated]");
    }
    text
}
