use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{LazyLock, Mutex};
use tempfile::NamedTempFile;

const MAX_TTS_CHARS: usize = 4_000;

static TTS_PROCESS: LazyLock<Mutex<Option<std::process::Child>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Serialize)]
pub struct TtsResult {
    pub base64: String,
    pub mime: String,
    pub detected_lang: Option<String>,
    pub kokoro_lang: Option<String>,
}

#[derive(Serialize)]
pub struct KokoroDetectResult {
    pub found: bool,
    pub path: Option<String>,
}

pub fn detect_kokoro(custom_path: &str) -> KokoroDetectResult {
    if let Some(path) = resolve_kokoro_path(custom_path) {
        return KokoroDetectResult {
            found: true,
            path: Some(path),
        };
    }
    KokoroDetectResult {
        found: false,
        path: None,
    }
}

pub fn cancel_synthesis() {
    if let Ok(mut guard) = TTS_PROCESS.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn synthesize(
    text: &str,
    kokoro_path: &str,
    voice: &str,
    speed: f32,
    auto_detect_language: bool,
) -> Result<TtsResult, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Nothing to speak".into());
    }

    let executable = resolve_kokoro_path(kokoro_path)
        .ok_or_else(|| "Kokoro not found. Set the path in Settings → Speech.".to_string())?;

    let spoken = truncate_for_tts(trimmed);
    let (detected_lang, kokoro_lang) = if auto_detect_language {
        let detected = detect_language(&spoken, &executable);
        let code = kokoro_lang_code(&detected);
        (Some(detected), Some(code.to_string()))
    } else {
        (None, None)
    };

    let output = NamedTempFile::new().map_err(|e| format!("Temp file failed: {e}"))?;
    let output_path = output.path().to_path_buf();
    let speed_str = format!("{speed:.2}");
    let voice = if voice.trim().is_empty() {
        "af_heart".to_string()
    } else {
        voice.trim().to_string()
    };

    let mut cmd = Command::new(&executable);
    cmd.arg("-m").arg(&voice).arg("-t").arg(&spoken).arg("-s").arg(&speed_str);

    if let Some(ref lang) = kokoro_lang {
        cmd.arg("-l").arg(lang);
    }

    cmd.arg("-o").arg(&output_path);

    cancel_synthesis();

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to run Kokoro ({executable}): {e}"))?;

    {
        let mut guard = TTS_PROCESS
            .lock()
            .map_err(|e| format!("TTS lock failed: {e}"))?;
        *guard = Some(child);
    }

    let wait_result = {
        let mut guard = TTS_PROCESS
            .lock()
            .map_err(|e| format!("TTS lock failed: {e}"))?;
        match guard.as_mut() {
            Some(child) => child
                .wait()
                .map_err(|e| format!("Kokoro wait failed: {e}")),
            None => return Err("TTS cancelled".into()),
        }
    };

    {
        let mut guard = TTS_PROCESS
            .lock()
            .map_err(|e| format!("TTS lock failed: {e}"))?;
        *guard = None;
    }

    let status = wait_result?;
    if !status.success() {
        if status.code() == Some(9) {
            return Err("TTS cancelled".into());
        }
        return Err(format!(
            "Kokoro exited with status {}",
            status.code().unwrap_or(-1)
        ));
    }

    let bytes = std::fs::read(&output_path).map_err(|e| format!("Read WAV failed: {e}"))?;
    if bytes.is_empty() {
        return Err("Kokoro produced an empty audio file".into());
    }

    Ok(TtsResult {
        base64: STANDARD.encode(bytes),
        mime: "audio/wav".to_string(),
        detected_lang,
        kokoro_lang,
    })
}

fn detect_language(text: &str, kokoro_path: &str) -> String {
    let python = match resolve_python_for_kokoro(kokoro_path) {
        Some(path) => path,
        None => return "en".to_string(),
    };

    let script = detect_lang_script_path();
    if !script.is_file() {
        return "en".to_string();
    }

    let sample = truncate_for_detection(text);
    let mut child = match Command::new(&python)
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return "en".to_string(),
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(sample.as_bytes());
    }

    let output = match child.wait_with_output() {
        Ok(output) if output.status.success() => output,
        _ => return "en".to_string(),
    };

    let lang = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
    if lang.is_empty() {
        "en".to_string()
    } else {
        lang
    }
}

fn truncate_for_detection(text: &str) -> String {
    if text.chars().count() <= 500 {
        return text.to_string();
    }
    text.chars().take(500).collect()
}

fn detect_lang_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("detect_lang.py")
}

fn kokoro_lang_code(iso639: &str) -> &'static str {
    match iso639.split('-').next().unwrap_or(iso639).trim().to_lowercase().as_str() {
        "fr" => "f",
        "en" => "a",
        "es" => "e",
        "it" => "i",
        "pt" => "p",
        "ja" => "j",
        "hi" => "h",
        "zh" => "z",
        _ => "a",
    }
}

fn resolve_python_for_kokoro(kokoro_path: &str) -> Option<String> {
    let kokoro = Path::new(kokoro_path);
    if let Some(bin_dir) = kokoro.parent() {
        for name in ["python3", "python"] {
            let candidate = bin_dir.join(name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }

    for cmd in ["python3", "python"] {
        if let Ok(output) = Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {cmd} 2>/dev/null"))
            .output()
        {
            if output.status.success() {
                let found = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if Path::new(&found).is_file() {
                    return Some(found);
                }
            }
        }
    }

    None
}

fn truncate_for_tts(text: &str) -> String {
    let mut out = String::with_capacity(text.len().min(MAX_TTS_CHARS));
    for ch in text.chars() {
        if out.len() >= MAX_TTS_CHARS {
            out.push_str("…");
            break;
        }
        out.push(ch);
    }
    out
}

fn resolve_kokoro_path(custom_path: &str) -> Option<String> {
    if let Some(path) = check_executable(custom_path) {
        return Some(path);
    }

    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("command -v kokoro 2>/dev/null")
        .output()
    {
        if output.status.success() {
            let found = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Some(path) = check_executable(&found) {
                return Some(path);
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let candidates = [
            format!("{home}/.local/bin/kokoro"),
            format!("{home}/.venv/bin/kokoro"),
            format!("{home}/Documents/non-ollama/.venv/bin/kokoro"),
        ];
        for candidate in candidates {
            if let Some(path) = check_executable(&candidate) {
                return Some(path);
            }
        }
    }

    None
}

fn check_executable(path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    let expanded = expand_home(trimmed);
    let p = Path::new(&expanded);
    if p.is_file() {
        return Some(expanded);
    }
    None
}

fn expand_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_text() {
        let long = "a".repeat(5000);
        assert_eq!(truncate_for_tts(&long).len(), MAX_TTS_CHARS + "…".len());
    }

    #[test]
    fn maps_kokoro_langs() {
        assert_eq!(kokoro_lang_code("fr"), "f");
        assert_eq!(kokoro_lang_code("en"), "a");
        assert_eq!(kokoro_lang_code("de"), "a");
    }
}
