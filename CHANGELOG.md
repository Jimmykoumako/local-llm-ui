# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-06-10

### Added

- Configurable Ollama URL (Settings → Connection, `OLLAMA_HOST` env)
- First-run setup modal when Ollama is unreachable
- Agent tools: filesystem, git, and terminal with approval flow
- Kokoro TTS read-aloud, multi-conversation UI, tagging, and export
- Cross-platform release CI (Windows, Linux, macOS) via `v*` tags
- [docs/RELEASE.md](./docs/RELEASE.md) install guide

### Changed

- README updated for cross-platform desktop distribution (BYO Ollama)
- CI runs `cargo check` on Linux, Windows, and macOS

### Fixed

- Windows agent terminal uses `cmd /C` instead of `sh -c`
- Workspace roots and Kokoro detection use `USERPROFILE` on Windows

## [0.1.0] - 2026-06-09

### Added

- Tauri 2 + SvelteKit desktop UI for local Ollama models
- Model list with capability badges (vision, tools, thinking, audio)
- Streaming chat with collapsible thinking section
- Image attachments for vision models
- Audio attachments with ffmpeg-first conversion and Rust fallback
- 30-second audio trim and expanded context for audio requests
- Tool call display (execution loop planned)

[0.2.0]: https://github.com/Jimmykoumako/local-llm-ui/releases/tag/v0.2.0
[0.1.0]: https://github.com/Jimmykoumako/local-llm-ui/releases/tag/v0.1.0
