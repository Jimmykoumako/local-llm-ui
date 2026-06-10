use std::path::Path;

#[cfg(windows)]
use std::path::PathBuf;
use std::process::Command;

pub fn home_dir() -> Option<String> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Some(home);
        }
    }

    if let Ok(profile) = std::env::var("USERPROFILE") {
        if !profile.is_empty() {
            return Some(profile);
        }
    }

    None
}

pub fn documents_dir() -> Option<String> {
    let home = home_dir()?;

    #[cfg(windows)]
    {
        let docs = PathBuf::from(&home).join("Documents");
        if docs.is_dir() {
            return Some(docs.display().to_string());
        }
    }

    #[cfg(not(windows))]
    {
        let docs = format!("{home}/Documents");
        if Path::new(&docs).is_dir() {
            return Some(docs);
        }
    }

    None
}

pub fn expand_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return format!("{home}/{rest}");
        }
    }
    path.to_string()
}

pub fn which_executable(name: &str) -> Option<String> {
    #[cfg(windows)]
    {
        let output = Command::new("where").arg(name).output().ok()?;
        if !output.status.success() {
            return None;
        }
        let found = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()?
            .trim()
            .to_string();
        if Path::new(&found).is_file() {
            return Some(found);
        }
        None
    }

    #[cfg(not(windows))]
    {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {name} 2>/dev/null"))
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let found = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if Path::new(&found).is_file() {
            return Some(found);
        }
        None
    }
}

pub fn kokoro_search_paths() -> Vec<String> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };

    #[cfg(windows)]
    {
        vec![
            format!("{home}\\.local\\bin\\kokoro.exe"),
            format!("{home}\\.venv\\Scripts\\kokoro.exe"),
        ]
    }

    #[cfg(not(windows))]
    {
        vec![
            format!("{home}/.local/bin/kokoro"),
            format!("{home}/.venv/bin/kokoro"),
        ]
    }
}
