<script lang="ts">
  import type { AppView, SavedConversation } from "$lib/conversations";

  interface Props {
    conversations: SavedConversation[];
    activeConversationId: string | null;
    currentView: AppView;
    connected: boolean;
    statusMessage: string;
    chatsExpanded: boolean;
    collapsed: boolean;
    streamingIds: string[];
    taggedCount?: number;
    onNewChat: () => void;
    onSelectConversation: (id: string) => void;
    onDeleteConversation: (id: string) => void;
    onNavigate: (view: AppView) => void;
    onToggleChats: () => void;
    onToggleCollapse: () => void;
  }

  let {
    conversations,
    activeConversationId,
    currentView,
    connected,
    statusMessage,
    chatsExpanded,
    collapsed,
    streamingIds,
    taggedCount = 0,
    onNewChat,
    onSelectConversation,
    onDeleteConversation,
    onNavigate,
    onToggleChats,
    onToggleCollapse,
  }: Props = $props();

  const sorted = $derived(
    [...conversations].sort((a, b) => b.updatedAt - a.updatedAt),
  );
</script>

<aside class="sidebar" class:collapsed>
  <div class="brand">
    <div class="logo">LLM</div>
    {#if !collapsed}
      <div class="brand-text">
        <h1>Local LLM UI</h1>
        <p class="status" class:online={connected} class:offline={!connected}>
          <span class="dot"></span>{statusMessage}
        </p>
      </div>
    {/if}
    <button
      type="button"
      class="collapse-btn"
      onclick={onToggleCollapse}
      aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
      title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
    >
      {collapsed ? "›" : "‹"}
    </button>
  </div>

  <button
    type="button"
    class="new-chat"
    onclick={onNewChat}
    title="New chat"
    aria-label="New chat"
  >
    <span class="new-icon">+</span>
    {#if !collapsed}
      New chat
      <kbd class="kbd">Ctrl+N</kbd>
    {/if}
  </button>

  <nav class="nav">
    <button
      type="button"
      class="nav-item"
      class:active={currentView === "chat"}
      onclick={() => onNavigate("chat")}
      title="Chats"
    >
      <span class="nav-icon">💬</span>
      {#if !collapsed}<span class="nav-label">Chats</span>{/if}
    </button>
    <button
      type="button"
      class="nav-item"
      class:active={currentView === "tagged"}
      onclick={() => onNavigate("tagged")}
      title="Tagged messages"
    >
      <span class="nav-icon">★</span>
      {#if !collapsed}
        <span class="nav-label">Tagged</span>
        {#if taggedCount > 0}
          <span class="nav-count">{taggedCount}</span>
        {/if}
      {/if}
    </button>
    <button
      type="button"
      class="nav-item"
      class:active={currentView === "models"}
      onclick={() => onNavigate("models")}
      title="Models"
    >
      <span class="nav-icon">◇</span>
      {#if !collapsed}<span class="nav-label">Models</span>{/if}
    </button>
    <button
      type="button"
      class="nav-item"
      class:active={currentView === "settings"}
      onclick={() => onNavigate("settings")}
      title="Settings"
    >
      <span class="nav-icon">⚙</span>
      {#if !collapsed}<span class="nav-label">Settings</span>{/if}
    </button>
  </nav>

  {#if !collapsed}
    <div class="accordion">
      <button type="button" class="accordion-head" onclick={onToggleChats}>
        <span>Recent chats</span>
        <span class="count">{sorted.length}</span>
        <span class="chev" class:open={chatsExpanded}>▾</span>
      </button>

      {#if chatsExpanded}
        <ul class="chat-list">
          {#if sorted.length === 0}
            <li class="empty-item">No saved chats yet</li>
          {:else}
            {#each sorted as conv (conv.id)}
              <li class="chat-item" class:active={conv.id === activeConversationId}>
                <button
                  type="button"
                  class="chat-btn"
                  onclick={() => onSelectConversation(conv.id)}
                >
                  <span class="chat-title">
                    {conv.title}
                    {#if streamingIds.includes(conv.id)}
                      <span class="stream-dot" title="Generating…"></span>
                    {/if}
                  </span>
                  <span class="chat-meta">{conv.model}</span>
                </button>
                <button
                  type="button"
                  class="delete-btn"
                  onclick={() => onDeleteConversation(conv.id)}
                  aria-label="Delete chat"
                >×</button>
              </li>
            {/each}
          {/if}
        </ul>
      {/if}
    </div>
  {:else}
    <div class="collapsed-chats">
      {#each sorted.slice(0, 8) as conv (conv.id)}
        <button
          type="button"
          class="collapsed-chat-btn"
          class:active={conv.id === activeConversationId}
          onclick={() => onSelectConversation(conv.id)}
          title={conv.title}
          aria-label={conv.title}
        >
          <span class="collapsed-chat-icon">💬</span>
          {#if streamingIds.includes(conv.id)}
            <span class="stream-dot-mini"></span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}

  <div class="footer">
    {#if collapsed}
      <span
        class="footer-dot"
        class:online={connected}
        title={connected ? "Ollama connected" : "Offline"}
      ></span>
    {:else}
      <span class="footer-item">dev</span>
      <span class="footer-sep">·</span>
      <span class="footer-item">Local</span>
      <span class="footer-sep">·</span>
      <span class="footer-item" class:online={connected}>
        {connected ? "Ollama" : "Offline"}
      </span>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    background: var(--color-bg-elevated);
    border-right: 1px solid var(--color-border);
    overflow: hidden;
    min-height: 0;
    transition: padding 0.15s ease;
  }

  .sidebar.collapsed {
    padding: var(--space-3) var(--space-2);
    align-items: center;
  }

  .brand {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    flex-shrink: 0;
    width: 100%;
  }

  .sidebar.collapsed .brand {
    flex-direction: column;
    gap: var(--space-2);
  }

  .logo {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-md);
    background: var(--color-primary);
    font-weight: 700;
    font-size: var(--text-xs);
    flex-shrink: 0;
  }

  .brand-text {
    flex: 1;
    min-width: 0;
  }

  .brand h1 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 600;
  }

  .status {
    margin: 2px 0 0;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--color-text-muted);
  }

  .status.online .dot { background: var(--color-success); }
  .status.offline .dot { background: var(--color-error); }

  .collapse-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-inset);
    color: var(--color-text-muted);
    font-size: 1rem;
    flex-shrink: 0;
    margin-left: auto;
  }

  .sidebar.collapsed .collapse-btn {
    margin-left: 0;
  }

  .collapse-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .new-chat {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-inset);
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text);
    flex-shrink: 0;
  }

  .sidebar.collapsed .new-chat {
    width: 40px;
    height: 40px;
    padding: 0;
    justify-content: center;
  }

  .new-chat:hover {
    background: var(--color-bg-hover);
    border-color: var(--color-primary);
  }

  .new-icon {
    font-size: 1.1rem;
    line-height: 1;
  }

  .kbd {
    margin-left: auto;
    font-size: 0.65rem;
    padding: 2px 5px;
    border-radius: 4px;
    background: var(--color-bg-panel);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-shrink: 0;
    width: 100%;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    text-align: left;
    width: 100%;
  }

  .sidebar.collapsed .nav-item {
    justify-content: center;
    padding: var(--space-2);
    width: 40px;
  }

  .nav-item:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .nav-item.active {
    background: rgba(79, 106, 245, 0.12);
    color: var(--color-text);
  }

  .nav-icon {
    width: 18px;
    text-align: center;
    opacity: 0.8;
    flex-shrink: 0;
  }

  .nav-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nav-count {
    margin-left: auto;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    background: var(--color-bg-panel);
    padding: 1px 6px;
    border-radius: var(--radius-full);
  }

  .accordion {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    width: 100%;
  }

  .accordion-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-1);
    border: none;
    background: transparent;
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .count {
    font-weight: 400;
    opacity: 0.7;
  }

  .chev {
    margin-left: auto;
    transition: transform 0.15s;
  }

  .chev.open {
    transform: rotate(180deg);
  }

  .chat-list {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .empty-item {
    padding: var(--space-3);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .chat-item {
    display: flex;
    align-items: stretch;
    border-radius: var(--radius-sm);
    margin-bottom: 2px;
  }

  .chat-item.active {
    background: rgba(79, 106, 245, 0.1);
  }

  .chat-btn {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: var(--space-2) var(--space-3);
    border: none;
    background: transparent;
    text-align: left;
  }

  .chat-btn:hover {
    background: var(--color-bg-hover);
  }

  .chat-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    width: 100%;
  }

  .stream-dot {
    width: 7px;
    height: 7px;
    flex-shrink: 0;
    border-radius: var(--radius-full);
    background: var(--color-success);
    animation: pulse-stream 1.2s ease infinite;
  }

  @keyframes pulse-stream {
    50% { opacity: 0.35; }
  }

  .chat-meta {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    width: 100%;
  }

  .delete-btn {
    width: 28px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    opacity: 0;
    font-size: 1rem;
    flex-shrink: 0;
  }

  .chat-item:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    color: var(--color-error);
  }

  .collapsed-chats {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow-y: auto;
    width: 100%;
    align-items: center;
  }

  .collapsed-chat-btn {
    position: relative;
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
  }

  .collapsed-chat-btn:hover {
    background: var(--color-bg-hover);
  }

  .collapsed-chat-btn.active {
    background: rgba(79, 106, 245, 0.15);
  }

  .collapsed-chat-icon {
    font-size: 1rem;
    opacity: 0.85;
  }

  .stream-dot-mini {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--color-success);
    animation: pulse-stream 1.2s ease infinite;
  }

  .footer {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding-top: var(--space-3);
    border-top: 1px solid var(--color-border-subtle);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    width: 100%;
    justify-content: center;
  }

  .sidebar:not(.collapsed) .footer {
    justify-content: flex-start;
  }

  .footer-item.online {
    color: var(--color-success);
  }

  .footer-sep {
    opacity: 0.4;
  }

  .footer-dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--color-text-muted);
  }

  .footer-dot.online {
    background: var(--color-success);
  }
</style>
