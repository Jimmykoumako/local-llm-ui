# Local LLM UI

A lightweight Linux desktop app for chatting with [Ollama](https://ollama.com) models. Built with **Rust + Tauri 2** and **SvelteKit**.

## Features

- Connect to a local Ollama instance (`http://127.0.0.1:11434`)
- List installed models with capability badges (`vision`, `tools`, `thinking`)
- Stream chat responses in real time
- Show model **thinking** in a separate collapsible section
- Attach **images** for vision-capable models
- Attach **audio** for Gemma 4-style models (auto-converted to 16 kHz mono WAV via ffmpeg)
- Display **tool calls** returned by the model (execution loop coming next)

## Prerequisites (Linux)

1. **Rust** — install via [rustup](https://rustup.rs/):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

2. **Tauri Linux dependencies** — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#linux):

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

3. **Ollama** — running locally with at least one model pulled:

   ```bash
   ollama serve
   ollama pull qwen3
   ollama pull gemma4   # for audio input
   ```

4. **ffmpeg** (optional) — preferred for audio conversion; the app falls back to built-in Rust decoding if ffmpeg is missing:

   ```bash
   sudo apt install ffmpeg
   ```

5. **Node.js** — v18+ (you already have this)

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

The binary and `.deb`/`.AppImage` will be in `src-tauri/target/release/bundle/`.

## Versioning

This project uses [Semantic Versioning](https://semver.org/) and documents releases in [CHANGELOG.md](./CHANGELOG.md).

See [docs/BRANCHING.md](./docs/BRANCHING.md) for the branch workflow (`main` = production, `dev` = integration, `feature/*` = work in progress).

Planned features: [docs/ROADMAP.md](./docs/ROADMAP.md) · UI mockup prompts: [docs/stitch-prompts.md](./docs/stitch-prompts.md)

```bash
# Tag a release after updating version in package.json, Cargo.toml, and CHANGELOG
git tag -a v0.1.0 -m "v0.1.0"
git push origin v0.1.0
```

## Architecture

```
Svelte UI  --invoke/listen-->  Rust (Tauri)  --HTTP-->  Ollama API
```

- `ollama_list_models` — fetches `/api/tags` (+ `/api/show` for capabilities)
- `ollama_chat` — streams `/api/chat` and emits `chat-chunk` events
- `prepare_audio_for_ollama` — converts audio to 16 kHz mono WAV (ffmpeg first, Rust fallback), trims to 30s
- Frontend accumulates `thinking`, `content`, and `tool_calls` per message

### Audio input

Ollama accepts audio through the `images` field for models like **Gemma 4**. The app:

1. Lets you attach `.mp3`, `.wav`, `.ogg`, etc.
2. Converts to **16 kHz mono WAV** using **ffmpeg** when available, otherwise **built-in Rust** (symphonia)
3. Trims clips longer than **30 seconds** (Gemma 4 limit)
4. Sends the WAV as base64 in `images` with `num_ctx: 8192`
5. Defaults the prompt to *"Transcribe this audio"* if you only attach audio

## Next steps

- Tool execution agent loop (run tools, send `role: "tool"` results back)
- Model settings (temperature, system prompt)
- Chat history persistence
- Custom Ollama host/port setting
