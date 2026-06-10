# Product roadmap

Planned features for Local LLM UI. See [stitch-prompts.md](./stitch-prompts.md) for UI design prompts.

## v0.2.0 — Chat quality & control

| Feature | Description | Priority |
|---------|-------------|----------|
| **Stop streaming** | Cancel in-flight Ollama generation; abort fetch/stream on Rust side | P0 |
| **Processing time** | Show time-to-first-token and total generation duration per message | P0 |
| **Markdown preview** | Render assistant replies (code blocks, lists, links) instead of raw text | P0 |
| **Save conversation** | Persist chats to disk (JSON/SQLite); load and resume | P0 |

## v0.3.0 — Settings & configuration

| Feature | Description | Priority |
|---------|-------------|----------|
| **Settings view** | Dedicated screen: Ollama host/port, default model, context size, thinking toggle, theme | P0 |
| **Custom Ollama URL** | Support remote or non-default Ollama instances | P1 |
| **Default system prompt** | Editable system message applied to new chats | P1 |

## v0.4.0 — Multimodal output

| Feature | Description | Priority |
|---------|-------------|----------|
| **Generated image display** | When model returns image data (experimental Ollama image gen), show inline preview | P0 |
| **Image gallery panel** | Grid of generated images in current session | P2 |

## Future views (suggested)

| View | Why |
|------|-----|
| **Conversations library** | Browse, search, rename, delete saved chats |
| **Model explorer** | Pull/delete models, view capabilities, size, VRAM hints |
| **Tools & MCP panel** | Configure tools, run agent loop, MCP server connections |
| **Prompt templates** | Reusable prompts (transcribe, summarize, code review) |
| **Performance monitor** | Tokens/sec, model load time, memory (from Ollama stats) |
| **Compare models** | Same prompt to two models side-by-side |

## Technical notes

### Stop streaming

- Rust: use `reqwest` cancel token or drop stream on Tauri command
- Frontend: `AbortController` pattern via `invoke('ollama_chat_cancel')`
- UI: Stop button visible while `isStreaming`

### Save conversation

- Store: `~/.config/local-llm-ui/conversations/{uuid}.json`
- Schema: `{ id, title, model, createdAt, messages[] }`
- Auto-title from first user message

### Processing time

- Ollama returns `total_duration`, `eval_duration` in final chunk
- Display: `1.2s · 42 tok/s` under assistant messages

### Markdown preview

- Use `marked` + `dompurify` or `svelte-markdown`
- Toggle: raw / rendered per message or global setting

### Generated images

- Ollama experimental image API returns base64 in response
- Detect `images` on assistant message; render `<img>` inline

### Settings view

- New route: `/settings` or slide-over panel
- Persist with Tauri `store` plugin or `localStorage` + Rust file

## Stitch MCP

UI mockups for each view: [stitch-prompts.md](./stitch-prompts.md)
