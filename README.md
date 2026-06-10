# Local LLM UI

A cross-platform desktop app for chatting with [Ollama](https://ollama.com) models. Built with **Rust + Tauri 2** and **SvelteKit**.

**You install Ollama yourself** — this app is the UI and agent layer only.

## Features

- Connect to your Ollama instance (default `http://127.0.0.1:11434`, configurable)
- List installed models with capability badges (`vision`, `tools`, `thinking`)
- Stream chat responses in real time
- Show model **thinking** in a separate collapsible section
- Attach **images** for vision-capable models
- Attach **audio** for Gemma 4-style models (auto-converted to 16 kHz mono WAV via ffmpeg)
- **Agent tools** — filesystem, git, and terminal access with approvals
- **Kokoro TTS** — optional read-aloud (desktop)
- Multiple conversations, tagging, and export

## Quick start (end users)

1. Install [Ollama](https://ollama.com/download) and pull a model:
   ```bash
   ollama pull qwen3
   ```
2. Download the latest installer for your OS from [GitHub Releases](https://github.com/Jimmykoumako/local-llm-ui/releases).
3. Open Local LLM UI and confirm **Settings → Connection**.

See [docs/RELEASE.md](./docs/RELEASE.md) for per-platform notes.

## Prerequisites (development)

### All platforms

- **Rust** — [rustup](https://rustup.rs/)
- **Node.js** — v18+
- **Ollama** — running locally

### Linux

[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#linux):

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### Windows

- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/#windows)
- WebView2 (usually preinstalled on Windows 11)

### macOS

- Xcode Command Line Tools: `xcode-select --install`
- [Tauri macOS prerequisites](https://v2.tauri.app/start/prerequisites/#macos)

### Optional

- **ffmpeg** — preferred audio conversion; Rust fallback if missing
- **git** — agent git tools
- **Kokoro** — Settings → Speech

## Development

```bash
npm install
npm run tauri dev
```

Override Ollama URL for testing:

```bash
OLLAMA_HOST=http://127.0.0.1:11434 npm run tauri dev
```

## Build

```bash
npm run tauri build
```

Installers are written to `src-tauri/target/release/bundle/`.

## Releases

Tag-driven builds for Windows, Linux, and macOS:

```bash
git tag -a v0.1.0 -m "v0.1.0"
git push origin v0.1.0
```

Details: [docs/RELEASE.md](./docs/RELEASE.md)

## Versioning

[Semantic Versioning](https://semver.org/) · [CHANGELOG.md](./CHANGELOG.md) · [docs/BRANCHING.md](./docs/BRANCHING.md)

Planned features: [docs/ROADMAP.md](./docs/ROADMAP.md)

## Architecture

```
Svelte UI  --invoke/listen-->  Rust (Tauri)  --HTTP-->  Ollama API (user's machine)
                                    |
                            fs / git / shell / TTS
```

- `ollama_set_host` / `ollama_check` — configurable Ollama base URL
- `ollama_list_models` — `/api/tags` (+ `/api/show` for capabilities)
- `ollama_chat` — streams `/api/chat` and emits `chat-chunk` events
- `agent_execute_tool` — filesystem, git, and terminal tools inside allowed roots
- `prepare_audio_for_ollama` — 16 kHz mono WAV (ffmpeg first, Rust fallback), max 30s
