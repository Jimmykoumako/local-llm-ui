<script lang="ts">
  import {
    clearRememberedApprovals,
    forgetApproval,
    listRememberedApprovals,
  } from "$lib/agent-approvals";
  import type { AgentPermissionLevel, AgentSettings, ApprovalPolicy } from "$lib/agent-settings";
  import { approvalLabel, levelLabel } from "$lib/agent-settings";
  import {
    KOKORO_VOICE_SUGGESTIONS,
    type TtsSettings,
  } from "$lib/tts-settings";
  import ModelCapabilityAlert from "./ModelCapabilityAlert.svelte";
  import SettingsSection from "./SettingsSection.svelte";

  interface Props {
    thinkEnabled: boolean;
    supportsThinking: boolean;
    supportsVision: boolean;
    supportsAudio: boolean;
    supportsTools: boolean;
    connected: boolean;
    isStreaming: boolean;
    agentSettings: AgentSettings;
    onThinkChange: (value: boolean) => void;
    onRefresh: () => void;
    onAgentChange: (settings: AgentSettings) => void;
    onAddRoot: () => void;
    onRemoveRoot: (index: number) => void;
    selectedModel: string;
    toolSuggestions: string[];
    thinkingSuggestions: string[];
    maxConcurrentChats: number;
    onBrowseModels: () => void;
    onSelectModel: (name: string) => void;
    onMaxConcurrentChange: (value: number) => void;
    ttsSettings: TtsSettings;
    kokoroDetected: boolean;
    kokoroPath: string;
    ttsBusy: boolean;
    onTtsChange: (settings: TtsSettings) => void;
    onDetectKokoro: () => void;
    onTestTts: () => void;
    onStopTts: () => void;
  }

  const props: Props = $props();

  const thinkEnabled = $derived(props.thinkEnabled);
  const supportsThinking = $derived(props.supportsThinking);
  const supportsVision = $derived(props.supportsVision);
  const supportsAudio = $derived(props.supportsAudio);
  const supportsTools = $derived(props.supportsTools);
  const connected = $derived(props.connected);
  const isStreaming = $derived(props.isStreaming);
  const agentSettings = $derived(props.agentSettings);
  const selectedModel = $derived(props.selectedModel);
  const toolSuggestions = $derived(props.toolSuggestions);
  const thinkingSuggestions = $derived(props.thinkingSuggestions);
  const maxConcurrentChats = $derived(props.maxConcurrentChats);
  const ttsSettings = $derived(props.ttsSettings);
  const kokoroDetected = $derived(props.kokoroDetected);
  const kokoroPath = $derived(props.kokoroPath);
  const ttsBusy = $derived(props.ttsBusy);

  const levels: AgentPermissionLevel[] = ["off", "read", "write", "full"];
  const policies: ApprovalPolicy[] = ["always", "destructive", "session", "auto"];

  const SECTIONS = [
    {
      id: "connection",
      title: "Connection",
      keywords:
        "ollama connect connection status test localhost 11434 network",
    },
    {
      id: "agent",
      title: "Agent tools",
      keywords:
        "agent tools file git terminal shell workspace roots approval permission read write full folder",
    },
    {
      id: "chat",
      title: "Chat",
      keywords:
        "chat thinking traces concurrent parallel model streams",
    },
    {
      id: "speech",
      title: "Speech",
      keywords:
        "speech kokoro tts text voice read aloud auto speak detect",
    },
    {
      id: "multimodal",
      title: "Multimodal",
      keywords: "multimodal vision image audio input gemma",
    },
    {
      id: "storage",
      title: "Storage",
      keywords: "storage conversations save browser local",
    },
  ] as const;

  type SectionId = (typeof SECTIONS)[number]["id"];

  let remembered = $state<string[]>(listRememberedApprovals());
  let searchQuery = $state("");
  let activeSection = $state<SectionId>("connection");
  let openSections = $state<Record<SectionId, boolean>>({
    connection: true,
    agent: true,
    chat: true,
    speech: true,
    multimodal: true,
    storage: true,
  });

  let scrollRoot = $state<HTMLElement | null>(null);

  const normalizedQuery = $derived(searchQuery.trim().toLowerCase());

  const visibleSections = $derived.by(() => {
    if (!normalizedQuery) return SECTIONS;
    return SECTIONS.filter((section) => sectionMatches(section));
  });

  function sectionMatches(section: (typeof SECTIONS)[number]): boolean {
    if (!normalizedQuery) return true;
    const haystack = `${section.title} ${section.keywords} ${section.id}`.toLowerCase();
    return normalizedQuery
      .split(/\s+/)
      .filter(Boolean)
      .every((term) => haystack.includes(term));
  }

  function isSectionVisible(id: SectionId): boolean {
    return visibleSections.some((section) => section.id === id);
  }

  function toggleSection(id: SectionId) {
    openSections = { ...openSections, [id]: !openSections[id] };
  }

  function jumpToSection(id: SectionId) {
    openSections = { ...openSections, [id]: true };
    activeSection = id;
    requestAnimationFrame(() => {
      document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
    });
  }

  function refreshRemembered() {
    remembered = listRememberedApprovals();
  }

  function patch(partial: Partial<AgentSettings>) {
    props.onAgentChange({ ...agentSettings, ...partial });
  }

  function patchTts(partial: Partial<TtsSettings>) {
    props.onTtsChange({ ...ttsSettings, ...partial });
  }

  $effect(() => {
    if (!normalizedQuery) return;
    const next = { ...openSections };
    for (const section of SECTIONS) {
      if (sectionMatches(section)) next[section.id] = true;
    }
    openSections = next;
  });

  $effect(() => {
    const root = scrollRoot?.closest(".view-scroll") as HTMLElement | null;
    if (!root) return;

    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => b.intersectionRatio - a.intersectionRatio);
        const top = visible[0]?.target as HTMLElement | undefined;
        const id = top?.dataset.settingsSection as SectionId | undefined;
        if (id) activeSection = id;
      },
      { root, rootMargin: "-20% 0px -55% 0px", threshold: [0, 0.25, 0.5, 1] },
    );

    for (const section of SECTIONS) {
      const el = document.getElementById(section.id);
      if (el) observer.observe(el);
    }

    return () => observer.disconnect();
  });
</script>

<div class="settings-shell" bind:this={scrollRoot}>
  <div class="settings-main">
    <header class="view-header">
      <div class="header-top">
        <h2>Settings</h2>
        {#if normalizedQuery}
          <span class="search-count">{visibleSections.length} match{visibleSections.length === 1 ? "" : "es"}</span>
        {/if}
      </div>
      <label class="search">
        <span class="search-icon" aria-hidden="true">⌕</span>
        <input
          type="search"
          placeholder="Search settings…"
          bind:value={searchQuery}
          aria-label="Search settings"
        />
        {#if searchQuery}
          <button
            type="button"
            class="search-clear"
            onclick={() => (searchQuery = "")}
            aria-label="Clear search"
          >×</button>
        {/if}
      </label>
    </header>

    {#if visibleSections.length === 0}
      <p class="no-results">No settings match “{searchQuery}”.</p>
    {/if}

    <SettingsSection
      id="connection"
      title="Connection"
      open={openSections.connection}
      onToggle={() => toggleSection("connection")}
      hidden={!isSectionVisible("connection")}
    >
      <p class="desc">Ollama runs locally at <code>http://127.0.0.1:11434</code></p>
      <div class="row">
        <span>Status</span>
        <span class:ok={connected} class:bad={!connected}>
          {connected ? "Connected" : "Disconnected"}
        </span>
      </div>
      <button type="button" class="btn" onclick={props.onRefresh} disabled={isStreaming}>
        Test connection
      </button>
    </SettingsSection>

    <SettingsSection
      id="agent"
      title="Agent tools"
      open={openSections.agent}
      onToggle={() => toggleSection("agent")}
      highlight
      blocked={!supportsTools}
      hidden={!isSectionVisible("agent")}
    >
      {#if !supportsTools}
        <ModelCapabilityAlert
          title="This model cannot use agent tools"
          message="File, git, and terminal tools need a model with the tools capability. Pick one of the models below or open the Models view."
          suggestions={toolSuggestions}
          currentModel={selectedModel}
          onBrowseModels={props.onBrowseModels}
          onSelectModel={props.onSelectModel}
        />
      {:else}
        <p class="desc">
          Files, git, and terminal access inside allowed workspace folders.
          Destructive actions can require your approval first.
        </p>
      {/if}

      <div class="card-inner" class:dimmed={!supportsTools}>
        <label class="check">
          <input
            type="checkbox"
            checked={agentSettings.enabled}
            onchange={(e) => patch({ enabled: e.currentTarget.checked })}
            disabled={isStreaming || !supportsTools}
          />
          Enable agent tools
        </label>
        <label class="field">
          <span>Permission level</span>
          <select
            value={agentSettings.level}
            onchange={(e) =>
              patch({ level: e.currentTarget.value as AgentPermissionLevel })}
            disabled={!agentSettings.enabled || isStreaming}
          >
            {#each levels as level}
              <option value={level}>{levelLabel(level)}</option>
            {/each}
          </select>
        </label>

        <label class="check">
          <input
            type="checkbox"
            checked={agentSettings.gitEnabled}
            onchange={(e) => patch({ gitEnabled: e.currentTarget.checked })}
            disabled={!agentSettings.enabled || isStreaming}
          />
          Enable git tools (status, diff, commit, push…)
        </label>

        <label class="check">
          <input
            type="checkbox"
            checked={agentSettings.shellEnabled}
            onchange={(e) => patch({ shellEnabled: e.currentTarget.checked })}
            disabled={!agentSettings.enabled || isStreaming || agentSettings.level !== "full"}
          />
          Enable terminal (full access only, always asks approval)
        </label>
        {#if agentSettings.enabled && agentSettings.level !== "full"}
          <p class="hint">Terminal requires permission level “Full access”.</p>
        {/if}

        <label class="field">
          <span>Approval policy</span>
          <select
            value={agentSettings.approvalPolicy}
            onchange={(e) =>
              patch({ approvalPolicy: e.currentTarget.value as ApprovalPolicy })}
            disabled={!agentSettings.enabled || isStreaming}
          >
            {#each policies as policy}
              <option value={policy}>{approvalLabel(policy)}</option>
            {/each}
          </select>
        </label>

        <div class="roots">
          <div class="roots-head">
            <span>Allowed workspace roots</span>
            <button
              type="button"
              class="btn-sm"
              onclick={props.onAddRoot}
              disabled={!agentSettings.enabled || isStreaming}
            >
              Add folder
            </button>
          </div>
          {#if agentSettings.allowedRoots.length === 0}
            <p class="hint">No folders configured — defaults load on first enable.</p>
          {:else}
            <p class="hint roots-note">
              Only folders listed here are accessible to the agent. Add each test or project folder explicitly.
            </p>
            <ul class="root-list">
              {#each agentSettings.allowedRoots as root, i}
                <li>
                  <code>{root}</code>
                  <button
                    type="button"
                    class="remove"
                    onclick={() => props.onRemoveRoot(i)}
                    disabled={isStreaming}
                    aria-label="Remove folder"
                  >×</button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>

        <div class="perm-grid">
          <div>
            <strong>Read</strong>
            <span>files + git status/diff/log</span>
          </div>
          <div>
            <strong>Write</strong>
            <span>edit files + git add</span>
          </div>
          <div>
            <strong>Full</strong>
            <span>delete + git commit/push + shell</span>
          </div>
        </div>

        <div class="roots remembered">
          <div class="roots-head">
            <span>Remembered approvals</span>
            {#if remembered.length > 0}
              <button
                type="button"
                class="btn-sm"
                onclick={() => { clearRememberedApprovals(); refreshRemembered(); }}
                disabled={isStreaming}
              >
                Clear all
              </button>
            {/if}
          </div>
          {#if remembered.length === 0}
            <p class="hint">Use “Always allow for this file” in the approval dialog.</p>
          {:else}
            <ul class="root-list">
              {#each remembered as key}
                <li>
                  <code>{key}</code>
                  <button
                    type="button"
                    class="remove"
                    onclick={() => { forgetApproval(key); refreshRemembered(); }}
                    disabled={isStreaming}
                    aria-label="Remove remembered approval"
                  >×</button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      </div>
    </SettingsSection>

    <SettingsSection
      id="chat"
      title="Chat"
      open={openSections.chat}
      onToggle={() => toggleSection("chat")}
      blocked={!supportsThinking}
      hidden={!isSectionVisible("chat")}
    >
      {#if !supportsThinking}
        <ModelCapabilityAlert
          title="This model cannot show thinking traces"
          message="Thinking panels require a model with the thinking capability. Switch to a compatible model to enable this."
          suggestions={thinkingSuggestions}
          currentModel={selectedModel}
          onBrowseModels={props.onBrowseModels}
          onSelectModel={props.onSelectModel}
        />
      {/if}

      <div class="card-inner" class:dimmed={!supportsThinking}>
        <label class="check">
          <input
            type="checkbox"
            checked={thinkEnabled}
            onchange={(e) => props.onThinkChange(e.currentTarget.checked)}
            disabled={!supportsThinking || isStreaming}
          />
          Show thinking traces
        </label>
        {#if supportsThinking}
          <p class="hint">Thinking appears in a collapsible panel above replies.</p>
        {/if}

        <label class="field">
          <span>Max concurrent chats</span>
          <input
            type="number"
            min="1"
            max="8"
            value={maxConcurrentChats}
            onchange={(e) => {
              const n = Number.parseInt(e.currentTarget.value, 10);
              if (n >= 1 && n <= 8) props.onMaxConcurrentChange(n);
            }}
            disabled={isStreaming}
          />
        </label>
        <p class="hint">Run multiple chats in parallel (each streams in the background).</p>
      </div>
    </SettingsSection>

    <SettingsSection
      id="speech"
      title="Speech"
      open={openSections.speech}
      onToggle={() => toggleSection("speech")}
      blocked={!kokoroDetected}
      hidden={!isSectionVisible("speech")}
    >
      {#if !kokoroDetected}
        <p class="desc warn">
          Kokoro was not found on this system. Install it or set the path below, then click
          <strong>Detect Kokoro</strong>.
        </p>
      {:else}
        <p class="desc">
          Read assistant replies aloud using your local Kokoro install
          {#if kokoroPath}
            (<code>{kokoroPath}</code>).
          {/if}
        </p>
      {/if}

      <div class="card-inner" class:dimmed={!kokoroDetected}>
        <div class="row actions">
          <button
            type="button"
            class="btn"
            onclick={props.onDetectKokoro}
            disabled={isStreaming || ttsBusy}
          >
            Detect Kokoro
          </button>
          {#if ttsBusy}
            <button
              type="button"
              class="btn stop"
              onclick={props.onStopTts}
            >
              Stop
            </button>
          {:else}
            <button
              type="button"
              class="btn"
              onclick={props.onTestTts}
              disabled={isStreaming || !kokoroDetected}
            >
              Test voice
            </button>
          {/if}
        </div>

        <p class="hint">
          Language detection uses <code>langdetect</code> in Kokoro's Python environment:
          <code>pip install langdetect</code>
        </p>

        <label class="field">
          <span>Kokoro path (optional)</span>
          <input
            type="text"
            value={ttsSettings.kokoroPath}
            placeholder="Auto-detect or e.g. ~/.venv/bin/kokoro"
            onchange={(e) => patchTts({ kokoroPath: e.currentTarget.value.trim() })}
            disabled={isStreaming}
          />
        </label>

        <label class="field">
          <span>Voice</span>
          <input
            type="text"
            list="kokoro-voices"
            value={ttsSettings.voice}
            onchange={(e) => patchTts({ voice: e.currentTarget.value.trim() || "af_heart" })}
            disabled={isStreaming || !kokoroDetected}
          />
          <datalist id="kokoro-voices">
            {#each KOKORO_VOICE_SUGGESTIONS as voice}
              <option value={voice}></option>
            {/each}
          </datalist>
        </label>

        <label class="field">
          <span>Speed ({ttsSettings.speed.toFixed(2)}×)</span>
          <input
            type="range"
            min="0.6"
            max="1.6"
            step="0.05"
            value={ttsSettings.speed}
            oninput={(e) => patchTts({ speed: Number(e.currentTarget.value) })}
            disabled={isStreaming || !kokoroDetected}
          />
        </label>

        <label class="check">
          <input
            type="checkbox"
            checked={ttsSettings.autoDetectLanguage}
            onchange={(e) =>
              patchTts({ autoDetectLanguage: e.currentTarget.checked })}
            disabled={isStreaming || !kokoroDetected}
          />
          Auto-detect language (French → Kokoro <code>f</code>, English → <code>a</code>)
        </label>

        <label class="check">
          <input
            type="checkbox"
            checked={ttsSettings.enabled}
            onchange={(e) => patchTts({ enabled: e.currentTarget.checked })}
            disabled={isStreaming || !kokoroDetected}
          />
          Enable read-aloud on replies
        </label>

        <label class="check">
          <input
            type="checkbox"
            checked={ttsSettings.autoSpeak}
            onchange={(e) => patchTts({ autoSpeak: e.currentTarget.checked })}
            disabled={isStreaming || !kokoroDetected || !ttsSettings.enabled}
          />
          Automatically speak new assistant replies
        </label>
        <p class="hint">Each message also has a Speak button when TTS is enabled.</p>
      </div>
    </SettingsSection>

    <SettingsSection
      id="multimodal"
      title="Multimodal"
      open={openSections.multimodal}
      onToggle={() => toggleSection("multimodal")}
      hidden={!isSectionVisible("multimodal")}
    >
      <div class="row">
        <span>Vision (images)</span>
        <span class:ok={supportsVision}>{supportsVision ? "Supported" : "Off"}</span>
      </div>
      <div class="row">
        <span>Audio input</span>
        <span class:ok={supportsAudio}>{supportsAudio ? "30s max" : "Off"}</span>
      </div>
    </SettingsSection>

    <SettingsSection
      id="storage"
      title="Storage"
      open={openSections.storage}
      onToggle={() => toggleSection("storage")}
      hidden={!isSectionVisible("storage")}
    >
      <p class="desc">Conversations are saved automatically in your browser.</p>
    </SettingsSection>
  </div>

  <aside class="settings-nav" aria-label="Settings sections">
    <p class="nav-label">On this page</p>
    <nav>
      <ul>
        {#each SECTIONS as section (section.id)}
          {#if isSectionVisible(section.id)}
            <li>
              <button
                type="button"
                class="nav-link"
                class:active={activeSection === section.id}
                onclick={() => jumpToSection(section.id)}
              >
                {section.title}
              </button>
            </li>
          {/if}
        {/each}
      </ul>
    </nav>
  </aside>
</div>

<style>
  .settings-shell {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 148px;
    gap: var(--space-5);
    max-width: 860px;
    margin: 0 auto;
    padding: var(--space-6) var(--space-4);
    align-items: start;
  }

  .settings-main {
    min-width: 0;
  }

  .view-header {
    margin-bottom: var(--space-4);
  }

  .header-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .view-header h2 {
    margin: 0;
    font-size: var(--text-lg);
  }

  .search-count {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .search {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-inset);
  }

  .search-icon {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .search input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: var(--text-sm);
    outline: none;
    min-width: 0;
  }

  .search input::placeholder {
    color: var(--color-text-muted);
  }

  .search-clear {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 1.1rem;
    line-height: 1;
    padding: 0;
  }

  .no-results {
    margin: 0 0 var(--space-4);
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: 1px dashed var(--color-border);
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    text-align: center;
  }

  .settings-nav {
    position: sticky;
    top: var(--space-4);
    padding: var(--space-3);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-md);
    background: var(--color-bg-panel);
  }

  .nav-label {
    margin: 0 0 var(--space-2);
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
  }

  .settings-nav ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-link {
    width: 100%;
    border: none;
    background: transparent;
    text-align: left;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    line-height: 1.3;
  }

  .nav-link:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .nav-link.active {
    background: rgba(79, 106, 245, 0.14);
    color: var(--color-text);
    font-weight: 500;
  }

  .card-inner.dimmed {
    opacity: 0.45;
    pointer-events: none;
    user-select: none;
  }

  .card-inner {
    margin-top: var(--space-2);
  }

  .field input[type="number"] {
    width: 5rem;
  }

  .desc {
    margin: 0 0 var(--space-3);
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .row {
    display: flex;
    justify-content: space-between;
    padding: var(--space-2) 0;
    font-size: var(--text-sm);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .row:last-of-type {
    border-bottom: none;
    margin-bottom: var(--space-3);
  }

  .ok { color: var(--color-success); }
  .bad { color: var(--color-error); }

  .check {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    cursor: pointer;
    margin-bottom: var(--space-3);
  }

  .check input[type="checkbox"] {
    width: 1rem;
    height: 1rem;
    margin: 0;
    flex-shrink: 0;
    accent-color: var(--color-primary);
    cursor: pointer;
  }

  .check input[type="checkbox"]:disabled {
    cursor: not-allowed;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin-bottom: var(--space-3);
    font-size: var(--text-sm);
  }

  .field select,
  .field input[type="text"],
  .field input[type="number"] {
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-bg-inset);
    color: var(--color-text);
    font-size: var(--text-sm);
  }

  .actions {
    gap: var(--space-2);
    margin-bottom: var(--space-3);
  }

  .desc.warn {
    color: var(--color-warning);
  }

  .hint {
    margin: var(--space-2) 0 0;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .roots-note {
    margin: 0 0 var(--space-2);
  }

  .btn {
    padding: var(--space-2) var(--space-4);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-inset);
    font-size: var(--text-sm);
  }

  .btn.stop {
    color: var(--color-error);
    border-color: rgba(248, 113, 113, 0.45);
  }

  .btn-sm {
    padding: 2px 8px;
    font-size: var(--text-xs);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-inset);
  }

  .roots {
    margin-top: var(--space-2);
  }

  .remembered {
    margin-top: var(--space-4);
    padding-top: var(--space-3);
    border-top: 1px solid var(--color-border-subtle);
  }

  .roots-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--text-sm);
    margin-bottom: var(--space-2);
  }

  .root-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .root-list li {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    background: var(--color-bg-inset);
    border-radius: var(--radius-sm);
  }

  .root-list code {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .remove {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 1rem;
    padding: 0 4px;
  }

  .remove:hover {
    color: var(--color-error);
  }

  .perm-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-2);
    margin-top: var(--space-3);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .perm-grid strong {
    display: block;
    color: var(--color-text-secondary);
    margin-bottom: 2px;
  }

  code {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  @media (max-width: 760px) {
    .settings-shell {
      grid-template-columns: 1fr;
    }

    .settings-nav {
      position: static;
      order: -1;
    }

    .settings-nav ul {
      flex-direction: row;
      flex-wrap: wrap;
    }

    .nav-link {
      width: auto;
    }
  }
</style>
