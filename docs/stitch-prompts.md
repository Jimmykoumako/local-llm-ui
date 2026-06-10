# Stitch MCP design prompts

Use these prompts in [Stitch](https://stitch.withgoogle.com/) (or Stitch MCP) to generate UI mockups for Local LLM UI. Each prompt assumes a **dark desktop app** (Tauri), **1100×750** min window, sidebar + main content layout.

---

## Global design system (prepend to any prompt)

```
Design a dark-mode desktop chat application called "Local LLM UI" for Linux/macOS/Windows.

Design system:
- Background: #0f1117, panels: #151821 / #171b24
- Accent: #3d5afe (primary actions), success #7dcea0, error #ffb4b4
- Font: Inter or system UI, 14–16px body
- Border radius: 10–14px on cards and inputs
- Left sidebar: 280px, model picker, capability badges, status
- Main area: chat thread + composer footer
- Dense but readable; no gradients; minimal chrome
- Target: Tauri desktop app, not mobile web
```

---

## 1. Chat view (current + enhancements)

```
[Paste global design system above]

Design the main chat screen with these additions to a standard chat UI:

1. STOP button — red/outline, appears in composer while assistant is streaming (replace or sit next to Send)
2. Per-message metadata under assistant bubbles: "2.4s · 38 tok/s · gemma4:12b"
3. Assistant messages render Markdown (headings, code blocks with syntax highlight, lists) — NOT raw text
4. Toggle on each message: "Raw" | "Preview" for markdown
5. Collapsible "Thinking" section (purple tint) above assistant content when model supports thinking
6. Tool calls shown as compact cards with function name + JSON args (not full raw dump)
7. User attachments: image thumbnails + audio chips with duration label
8. Top of sidebar: green "Connected to Ollama" status dot

Show one complete thread: user message with image, assistant with thinking block + markdown code block + timing footer.
```

---

## 2. Settings view

```
[Paste global design system above]

Design a dedicated Settings screen for a local LLM desktop app. Navigation: sidebar item "Settings" or gear icon opens full-page settings (replacing chat) with back button.

Sections (grouped cards):

**Connection**
- Ollama host URL (default http://127.0.0.1:11434)
- Test connection button
- Connection status indicator

**Chat defaults**
- Default model (dropdown)
- Context size (num_ctx): 4096 / 8192 / 16384 / 32768
- Enable thinking by default (toggle)
- Default system prompt (multiline textarea)

**Appearance**
- Theme: Dark / Light / System
- Font size: S / M / L
- Markdown preview default: Preview / Raw

**Audio**
- Max clip duration (seconds, default 30)
- Prefer ffmpeg when available (toggle, info: "Rust fallback if unavailable")

**Storage**
- Conversations folder path (read-only + Open folder button)
- Auto-save conversations (toggle)

Footer: Save / Reset to defaults buttons.

Layout: settings form in main panel, sidebar still visible with Settings highlighted.
```

---

## 3. Conversations library view

```
[Paste global design system above]

Design a "Conversations" library view — list of saved chats.

Layout:
- Sidebar unchanged; main panel shows conversation list
- Header: "Conversations" + search box + "New chat" button
- List items: title (auto from first message), model name badge, date, message count, preview snippet (1 line)
- Hover: show delete icon, rename icon
- Empty state: illustration + "No saved conversations yet"
- Selected row highlight #1d2a44

Click row → opens that chat in main chat view.
Include 5 sample rows with varied titles ("Transcribe meeting notes", "Explain Rust ownership", etc.)
```

---

## 4. Generated image display (in chat)

```
[Paste global design system above]

Design assistant message bubbles that include AI-generated images from Ollama.

When the model returns an image:
- Show image inline below assistant text, max-width 512px, rounded corners
- Lightbox on click to view full size
- Caption: "Generated image" + optional prompt echo
- Download button on hover (top-right of image)
- If multiple images: horizontal scroll or 2-column grid

Show example thread where user asks "Generate a logo for a coffee shop" and assistant returns text + 512×512 image placeholder (use a warm brown/orange abstract square as placeholder).
```

---

## 5. Model explorer view

```
[Paste global design system above]

Design a "Models" explorer screen listing locally installed Ollama models.

Main panel:
- Header: "Models" + Refresh + "Pull model" (opens modal with model name input)
- Table/cards: model name, size, capability badges (vision, tools, thinking, audio), modified date
- Row actions: Chat (primary), Show details, Delete (destructive, confirm dialog)
- Running models section at top (if any): green dot + name + "Unload"

Detail drawer (slide from right):
- Full capability list, parameter size, quantization, template preview (monospace, truncated)
```

---

## 6. Tools & agent panel (future)

```
[Paste global design system above]

Design a slide-over or tab "Tools" panel for agentic chat.

Contents:
- List of registered tools (name, description, enabled toggle)
- "Add tool" — JSON schema editor or pick from presets (web search, calculator, file read)
- During chat: live log of tool executions — timeline with status icons (pending → running → done)
- Each tool result expandable (JSON or text)

Show split view: chat on left 60%, tool activity log on right 40% while agent is running.
```

---

## 7. Compare models view (future)

```
[Paste global design system above]

Design a side-by-side model comparison view.

User enters one prompt at top. Two columns below:
- Left column: model A dropdown + streaming response
- Right column: model B dropdown + streaming response
- Shared timing bar comparing total duration and tokens/sec
- Sync scroll option (toggle)

Use case: compare qwen3 vs gemma4 on same question.
Dark theme, equal column widths, divider between columns.
```

---

## MCP usage tip

When calling Stitch MCP, structure requests as:

```json
{
  "prompt": "<paste full prompt from section above>",
  "context": "Local LLM UI — Tauri 2 desktop app for Ollama",
  "viewport": { "width": 1100, "height": 750 },
  "platform": "desktop"
}
```

Generate one view per session; export assets/specs for implementation in SvelteKit routes under `src/routes/`.
