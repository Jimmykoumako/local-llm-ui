# Releasing Local LLM UI

Local LLM UI ships as a **native desktop app** for Windows, Linux, and macOS. Users install and run **Ollama separately** on their own machine.

## User setup (all platforms)

1. Install [Ollama](https://ollama.com/download).
2. Pull at least one model:
   ```bash
   ollama pull qwen3
   ```
3. Install the Local LLM UI build for your OS (see GitHub Releases).
4. Open the app. If Ollama is not detected, use the setup dialog or **Settings → Connection**.
5. Confirm the Ollama URL (default `http://127.0.0.1:11434`).

### Optional

- **ffmpeg** — better audio conversion for multimodal models.
- **git** — required for agent git tools.
- **Kokoro** — optional text-to-speech in Settings → Speech.

## Build artifacts

| OS | Typical outputs |
|----|-----------------|
| Linux | `.deb`, `.AppImage` |
| Windows | `.msi`, `.exe` |
| macOS | `.dmg`, `.app` |

Local build:

```bash
npm install
npm run tauri build
```

Artifacts appear under `src-tauri/target/release/bundle/`.

## CI releases

Pushing a version tag triggers `.github/workflows/release.yml`:

```bash
# Bump version in package.json, src-tauri/Cargo.toml, and CHANGELOG.md first
git tag -a v0.2.0 -m "v0.2.0"
git push origin v0.2.0
```

GitHub Actions builds on **ubuntu-22.04**, **windows-latest**, and **macos-latest**, then uploads installers to a **draft** GitHub Release. Review the draft, publish when ready.

## Platform notes

### Linux

- Ollama usually listens on `http://127.0.0.1:11434`.
- AppImage may need `chmod +x` before running.

### Windows

- Install Ollama for Windows; the tray app keeps the API running.
- Agent terminal tools use `cmd /C`.
- If Ollama runs in WSL, set the URL to the WSL address (e.g. `http://localhost:11434` or the WSL IP).

### macOS

- Install Ollama for macOS from the official site.
- Unsigned builds may require right-click → Open the first time.
- For public distribution, plan code signing and notarization later.

## Ollama URL configuration

- **In app:** Settings → Connection → Ollama URL → Save & test connection.
- **Environment variable:** `OLLAMA_HOST=http://127.0.0.1:11434` (applied on first launch before UI overrides).

## What we do not ship

- Ollama binaries or model weights
- Docker images for the UI (desktop-only distribution)
- A browser/web edition (desktop agent features require native access)
