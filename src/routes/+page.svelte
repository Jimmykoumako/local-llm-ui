<script lang="ts">
  import { tick } from "svelte";
  import {
    ensureAgentRoots,
    loadAgentSettings,
    saveAgentSettings,
    type AgentSettings,
  } from "$lib/agent-settings";
  import { rememberApproval } from "$lib/agent-approvals";
  import {
    approvePlanExecution,
    buildSingleStepExecutionPrompt,
    buildSingleStepUserMessage,
    buildPlanningSystemPrompt,
    buildPlanningUserMessage,
    isPlanExecutionApproved,
    allPlanStepsDone,
    autoPlanNotice,
    isPlanCompleteMarker,
    parsePlanFromContent,
    planContinueNudge,
    planReadySummary,
    planWriteSteps,
    shouldAutoPlan,
    type AgentPlan,
    type PlanStepStatus,
  } from "$lib/agent-plan";
  import {
    buildAgentSystemPrompt,
    executeAgentTool,
    isAgentTool,
    isPermissionDeniedResult,
    needsApproval,
    recordSessionApproval,
    toolsForLevel,
    toolsForPlanning,
    VISION_AGENT_SUPPLEMENT,
  } from "$lib/agent-tools";
  import { suggestModels } from "$lib/model-capabilities";
  import ToastStack from "$lib/components/ToastStack.svelte";
  import { detectKokoro, speakText, stopSpeech, subscribeSpeaking } from "$lib/tts";
  import {
    loadTtsSettings,
    saveTtsSettings,
    type TtsSettings,
  } from "$lib/tts-settings";
  import { pushToast } from "$lib/toast";
  import { open } from "@tauri-apps/plugin-dialog";
  import AppSidebar from "$lib/components/AppSidebar.svelte";
  import ChatBubble from "$lib/components/ChatBubble.svelte";
  import TaggedView from "$lib/components/TaggedView.svelte";
  import ChatComposer from "$lib/components/ChatComposer.svelte";
  import ModelsView from "$lib/components/ModelsView.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import PlanApprovalModal from "$lib/components/PlanApprovalModal.svelte";
  import PlanPanel from "$lib/components/PlanPanel.svelte";
  import ToolApprovalModal from "$lib/components/ToolApprovalModal.svelte";
  import WorkspacePromptModal from "$lib/components/WorkspacePromptModal.svelte";
  import OllamaSetupModal from "$lib/components/OllamaSetupModal.svelte";
  import { truncateHistoryToMessages } from "$lib/branch-conversation";
  import { copyToClipboard } from "$lib/utils/clipboard";
  import {
    buildChatMarkdown,
    downloadMarkdown,
    markdownFilename,
  } from "$lib/utils/download";
  import {
    isMessageTagged,
    loadTaggedMessages,
    messagePreview,
    removeTaggedMessage,
    saveTaggedMessages,
    toggleTaggedMessage,
    type TaggedMessage,
  } from "$lib/tagged-messages";
  import {
    applyModePrefix,
    deriveTitle,
    loadConversations,
    saveConversations,
    type AppView,
    type ChatMode,
    type SavedConversation,
  } from "$lib/conversations";
  import {
    cancelChat,
    checkOllama,
    formatAudioNote,
    hasCapability,
    listModels,
    prepareAudioForOllama,
    streamChat,
    supportsAudio,
  } from "$lib/ollama";
  import type {
    AttachmentDisplay,
    ChatMessage,
    DisplayMessage,
    ModelInfo,
    PendingAttachment,
    ToolCallDisplay,
  } from "$lib/types";
  import { levelLabel } from "$lib/agent-settings";
  import {
    loadAppPreferences,
    loadLastModel,
    pickAvailableModel,
    saveAppPreferences,
    saveLastModel,
  } from "$lib/preferences";
  import {
    applyOllamaHost,
    loadOllamaHost,
    loadOllamaSetupDismissed,
    saveOllamaSetupDismissed,
    syncOllamaHostFromStorage,
  } from "$lib/ollama-settings";
  import { wavBase64ToUrl } from "$lib/utils/audio";
  import { parseToolCalls, toolCallsForOllamaHistory } from "$lib/utils/tools";

  const DEFAULT_AUDIO_PROMPT = "Transcribe this audio";
  const MAX_TOOL_ROUNDS = 5;
  const PLAN_MAX_ROUNDS = 6;
  const STEP_MAX_ROUNDS = 8;
  const activeSessions = new Map<string, string>();
  const abortedConversations = new Set<string>();

  let models = $state<ModelInfo[]>([]);
  let selectedModel = $state(loadLastModel());
  let connected = $state(false);
  let statusMessage = $state("Checking Ollama…");
  let thinkEnabled = $state(true);
  let chatMode = $state<ChatMode>("ask");
  let input = $state("");
  let pending = $state<PendingAttachment[]>([]);
  let messages = $state<DisplayMessage[]>([]);
  let history = $state<ChatMessage[]>([]);
  let streamingIds = $state<Set<string>>(new Set());
  let appPreferences = $state(loadAppPreferences());
  let isPreparingAudio = $state(false);
  let errorMessage = $state("");
  let messagesEnd = $state<HTMLDivElement | null>(null);

  let conversations = $state<SavedConversation[]>(loadConversations());
  let activeConversationId = $state<string | null>(null);
  let currentView = $state<AppView>("chat");
  let chatsExpanded = $state(true);
  let agentSettings = $state<AgentSettings>(loadAgentSettings());
  let agentPlan = $state<AgentPlan | null>(null);
  let pendingApproval = $state<{
    tool: ToolCallDisplay;
    resolve: (ok: boolean) => void;
  } | null>(null);
  let pendingPlanApproval = $state<{
    plan: AgentPlan;
    resolve: (ok: boolean) => void;
  } | null>(null);
  let showWorkspacePrompt = $state(false);
  let ollamaHost = $state(loadOllamaHost());
  let showOllamaSetup = $state(false);
  let ollamaCheckBusy = $state(false);
  let ttsSettings = $state<TtsSettings>(loadTtsSettings());
  let kokoroDetected = $state(false);
  let kokoroPath = $state("");
  let ttsBusy = $state(false);
  let speakingTtsTarget = $state<string | null>(null);
  let taggedMessages = $state(loadTaggedMessages());
  let scrollToMessageId = $state<string | null>(null);

  const selectedModelInfo = $derived(
    models.find((m) => m.name === selectedModel),
  );
  const supportsThinking = $derived(
    hasCapability(selectedModelInfo, "thinking"),
  );
  const supportsVision = $derived(hasCapability(selectedModelInfo, "vision"));
  const supportsTools = $derived(hasCapability(selectedModelInfo, "tools"));
  const supportsAudioInput = $derived(supportsAudio(selectedModelInfo));
  const hasPendingAudio = $derived(
    pending.some((a) => a.kind === "audio"),
  );
  const chatTitle = $derived.by(() => {
    if (messages.length > 0) return deriveTitle(messages);
    const conv = conversations.find((c) => c.id === activeConversationId);
    return conv?.title ?? "New chat";
  });

  function exportChatMarkdown() {
    const exportable = messages.filter((m) => m.content.trim() && !m.streaming);
    if (exportable.length === 0) return;
    void downloadMarkdown(
      markdownFilename(chatTitle),
      buildChatMarkdown(
        exportable.map((m) => ({ role: m.role, content: m.content })),
      ),
    );
  }
  const isActiveStreaming = $derived(
    activeConversationId !== null && streamingIds.has(activeConversationId),
  );
  const streamingIdList = $derived([...streamingIds]);
  const toolModelSuggestions = $derived(suggestModels(models, "tools"));
  const thinkingModelSuggestions = $derived(suggestModels(models, "thinking"));
  function notifyToolsUnavailable(action: string) {
    const hint = toolModelSuggestions[0];
    pushToast(
      hint
        ? `Can't ${action} — ${selectedModel || "this model"} has no tools support. Try ${hint} in Settings → Models.`
        : `Can't ${action} — pick a tools-capable model in Settings → Models.`,
      "warning",
    );
  }

  function updateTtsSettings(next: TtsSettings) {
    ttsSettings = next;
    saveTtsSettings(next);
    void refreshKokoroDetect();
  }

  async function refreshKokoroDetect() {
    try {
      const result = await detectKokoro(ttsSettings.kokoroPath);
      kokoroDetected = result.found;
      kokoroPath = result.path ?? "";
    } catch {
      kokoroDetected = false;
      kokoroPath = "";
    }
  }

  function isTtsCancelled(error: unknown): boolean {
    return String(error).toLowerCase().includes("cancel");
  }

  function stopTts() {
    stopSpeech();
    speakingTtsTarget = null;
  }

  async function testTtsVoice() {
    if (!kokoroDetected) {
      pushToast("Kokoro not found. Set the path in Settings → Speech.", "warning");
      return;
    }
    try {
      await speakText(
        "Kokoro text to speech is working in Local LLM UI.",
        ttsSettings,
      );
    } catch (error) {
      if (!isTtsCancelled(error)) pushToast(String(error), "error");
    }
  }

  async function maybeAutoSpeak(text: string) {
    if (!ttsSettings.enabled || !ttsSettings.autoSpeak || !kokoroDetected) return;
    const trimmed = text.trim();
    if (!trimmed) return;
    try {
      await speakText(trimmed, ttsSettings);
    } catch (error) {
      if (!isTtsCancelled(error)) pushToast(String(error), "error");
    }
  }

  function speakingBlockKeyFor(messageId: string): string | null {
    const prefix = `${messageId}:`;
    if (!speakingTtsTarget?.startsWith(prefix)) return null;
    return speakingTtsTarget.slice(prefix.length);
  }

  async function speakMessage(messageId: string, text: string) {
    if (!ttsSettings.enabled) {
      pushToast("Enable read-aloud in Settings → Speech.", "warning");
      return;
    }
    if (!kokoroDetected) {
      pushToast("Kokoro not found. Configure it in Settings → Speech.", "warning");
      return;
    }
    speakingTtsTarget = messageId;
    try {
      await speakText(text, ttsSettings);
    } catch (error) {
      if (!isTtsCancelled(error)) pushToast(String(error), "error");
    } finally {
      if (!ttsBusy) speakingTtsTarget = null;
    }
  }

  function branchFromMessage(index: number) {
    if (!activeConversationId || index < 0 || index >= messages.length) return;
    syncActiveConversation(true);

    const branchMessages = messages.slice(0, index + 1).map((m) => ({
      ...m,
      streaming: false,
    }));
    const branchHistory = truncateHistoryToMessages(history, branchMessages);
    const newId = crypto.randomUUID();
    const baseTitle = deriveTitle(branchMessages);

    const newConv: SavedConversation = {
      id: newId,
      title: baseTitle === "New chat" ? "Branch" : `${baseTitle} (branch)`,
      model: selectedModel,
      thinkEnabled,
      chatMode,
      createdAt: Date.now(),
      updatedAt: Date.now(),
      messages: branchMessages,
      history: branchHistory,
      agentPlan: null,
    };

    conversations = [newConv, ...conversations];
    saveConversations(conversations);
    selectConversation(newId);
    pushToast("Branched chat from this message");
  }

  function toggleMessageTag(message: DisplayMessage) {
    if (!activeConversationId) return;
    taggedMessages = toggleTaggedMessage(taggedMessages, {
      conversationId: activeConversationId,
      messageId: message.id,
      conversationTitle: chatTitle,
      role: message.role,
      preview: messagePreview(message.content),
      content: message.content,
    });
    saveTaggedMessages(taggedMessages);
    const tagged = isMessageTagged(
      taggedMessages,
      activeConversationId,
      message.id,
    );
    pushToast(tagged ? "Message tagged" : "Tag removed");
  }

  function openTaggedMessage(tag: TaggedMessage) {
    const conv = conversations.find((c) => c.id === tag.conversationId);
    if (!conv) {
      taggedMessages = removeTaggedMessage(taggedMessages, tag.id);
      saveTaggedMessages(taggedMessages);
      pushToast("Original chat no longer exists — tag removed", "warning");
      return;
    }
    scrollToMessageId = tag.messageId;
    selectConversation(tag.conversationId);
  }

  function removeTaggedMessageById(id: string) {
    taggedMessages = removeTaggedMessage(taggedMessages, id);
    saveTaggedMessages(taggedMessages);
  }

  async function speakMessageBlock(
    messageId: string,
    blockKey: string,
    blockText: string,
  ) {
    if (!ttsSettings.enabled) {
      pushToast("Enable read-aloud in Settings → Speech.", "warning");
      return;
    }
    if (!kokoroDetected) {
      pushToast("Kokoro not found. Configure it in Settings → Speech.", "warning");
      return;
    }
    speakingTtsTarget = `${messageId}:${blockKey}`;
    try {
      await speakText(blockText, ttsSettings, { raw: true });
    } catch (error) {
      if (!isTtsCancelled(error)) pushToast(String(error), "error");
    } finally {
      if (!ttsBusy) speakingTtsTarget = null;
    }
  }

  async function refreshModels() {
    ollamaCheckBusy = true;
    try {
      ollamaHost = await applyOllamaHost(ollamaHost);
      await checkOllama();
      connected = true;
      statusMessage = "Connected to Ollama";
      models = await listModels();
      if (models.length > 0) {
        const next = pickAvailableModel(
          models,
          selectedModel,
          loadLastModel(),
        );
        setSelectedModel(next);
      }
      errorMessage = "";
      showOllamaSetup = false;
    } catch (error) {
      connected = false;
      statusMessage = "Ollama not reachable";
      errorMessage = String(error);
      models = [];
      if (!loadOllamaSetupDismissed()) {
        showOllamaSetup = true;
      }
    } finally {
      ollamaCheckBusy = false;
    }
  }

  async function retryOllamaSetup() {
    await refreshModels();
    if (connected) {
      saveOllamaSetupDismissed(true);
    }
  }

  function dismissOllamaSetup() {
    showOllamaSetup = false;
    saveOllamaSetupDismissed(true);
  }

  function openOllamaSettings() {
    showOllamaSetup = false;
    currentView = "settings";
  }

  function syncActiveConversation(finalize = false) {
    if (!activeConversationId) return;

    const existing = conversations.find((c) => c.id === activeConversationId);
    const nextMessages = finalize
      ? messages.map((m) => ({ ...m, streaming: false }))
      : [...messages];

    const conv: SavedConversation = {
      id: activeConversationId,
      title: messages.length > 0 ? deriveTitle(messages) : (existing?.title ?? "New chat"),
      model: selectedModel,
      thinkEnabled,
      chatMode,
      createdAt: existing?.createdAt ?? Date.now(),
      updatedAt: Date.now(),
      messages: nextMessages,
      history: [...history],
      agentPlan,
    };

    if (existing) {
      conversations = conversations.map((c) =>
        c.id === activeConversationId ? conv : c,
      );
    } else if (messages.length > 0) {
      conversations = [conv, ...conversations];
    }
    saveConversations(conversations);
    updateAppPreferences({ lastActiveConversationId: activeConversationId });
  }

  function persistCurrentChat() {
    if (!activeConversationId || messages.length === 0) return;
    syncActiveConversation(true);
  }

  function getConversationMessages(convId: string): DisplayMessage[] {
    if (convId === activeConversationId) return messages;
    return conversations.find((c) => c.id === convId)?.messages ?? [];
  }

  function setConversationMessages(convId: string, next: DisplayMessage[]) {
    if (convId === activeConversationId) messages = next;
    conversations = conversations.map((c) =>
      c.id === convId ? { ...c, messages: next, updatedAt: Date.now() } : c,
    );
  }

  function setConversationHistory(convId: string, next: ChatMessage[]) {
    if (convId === activeConversationId) history = next;
    conversations = conversations.map((c) =>
      c.id === convId ? { ...c, history: next } : c,
    );
  }

  function setConversationPlan(convId: string, plan: AgentPlan | null) {
    if (convId === activeConversationId) agentPlan = plan;
    conversations = conversations.map((c) =>
      c.id === convId ? { ...c, agentPlan: plan, updatedAt: Date.now() } : c,
    );
    saveConversations(conversations);
  }

  function patchPlanStep(
    convId: string,
    stepIndex: number,
    status: PlanStepStatus,
  ) {
    const plan = convId === activeConversationId
      ? agentPlan
      : conversations.find((c) => c.id === convId)?.agentPlan;
    if (!plan || stepIndex < 0 || stepIndex >= plan.steps.length) return;

    const steps = plan.steps.map((s, i) =>
      i === stepIndex ? { ...s, status } : s,
    );
    setConversationPlan(convId, { ...plan, steps });
  }

  function appendConvMessage(convId: string, msg: DisplayMessage) {
    setConversationMessages(convId, [...getConversationMessages(convId), msg]);
  }

  function contextToolCallsForMessage(index: number): ToolCallDisplay[] {
    let lastUserIdx = -1;
    for (let i = index; i >= 0; i--) {
      if (messages[i].role === "user") {
        lastUserIdx = i;
        break;
      }
    }

    const tools: ToolCallDisplay[] = [];
    for (let i = lastUserIdx + 1; i <= index; i++) {
      const roundTools = messages[i].toolCalls;
      if (roundTools?.length) {
        tools.push(...roundTools);
      }
    }
    return tools;
  }

  function patchConvMessage(
    convId: string,
    id: string,
    patch: Partial<DisplayMessage>,
  ) {
    setConversationMessages(
      convId,
      getConversationMessages(convId).map((m) =>
        m.id === id ? { ...m, ...patch } : m,
      ),
    );
  }

  async function stopStreaming() {
    const convId = activeConversationId;
    if (!convId) return;

    abortedConversations.add(convId);
    const sessionId = activeSessions.get(convId);

    if (sessionId) {
      try {
        await cancelChat(sessionId);
      } catch {
        // Stream may have just finished.
      }
    }

    activeSessions.delete(convId);
    removeStreaming(convId);

    setConversationMessages(
      convId,
      getConversationMessages(convId).map((m) =>
        m.streaming ? { ...m, streaming: false } : m,
      ),
    );
  }

  function toggleSidebarCollapsed() {
    updateAppPreferences({
      sidebarCollapsed: !appPreferences.sidebarCollapsed,
    });
  }

  function addStreaming(convId: string) {
    streamingIds = new Set([...streamingIds, convId]);
  }

  function removeStreaming(convId: string) {
    streamingIds = new Set([...streamingIds].filter((id) => id !== convId));
  }

  function startNewChat() {
    if (activeConversationId && messages.length > 0) {
      syncActiveConversation(true);
    }
    const newId = crypto.randomUUID();
    activeConversationId = newId;
    messages = [];
    history = [];
    agentPlan = null;
    input = "";
    pending = [];
    errorMessage = "";
    chatMode = "ask";
    currentView = "chat";
    updateAppPreferences({ lastActiveConversationId: newId });
  }

  function selectConversation(id: string) {
    if (activeConversationId && activeConversationId !== id) {
      syncActiveConversation(true);
    }
    const conv = conversations.find((c) => c.id === id);
    if (!conv) return;
    activeConversationId = id;
    messages = conv.messages;
    history = conv.history;
    agentPlan = conv.agentPlan ?? null;
    setSelectedModel(conv.model);
    thinkEnabled = conv.thinkEnabled;
    chatMode = conv.chatMode;
    input = "";
    pending = [];
    errorMessage = "";
    currentView = "chat";
    updateAppPreferences({ lastActiveConversationId: id });
  }

  function deleteConversation(id: string) {
    conversations = conversations.filter((c) => c.id !== id);
    saveConversations(conversations);
    if (activeConversationId === id) {
      startNewChat();
    }
  }

  function navigate(view: AppView) {
    currentView = view;
  }

  function setSelectedModel(name: string) {
    selectedModel = name;
    saveLastModel(name);
  }

  function updateAppPreferences(partial: Partial<typeof appPreferences>) {
    appPreferences = { ...appPreferences, ...partial };
    saveAppPreferences(appPreferences);
  }

  let conversationsHydrated = false;
  let connectionInitialized = false;

  $effect(() => {
    if (connectionInitialized) return;
    connectionInitialized = true;
    void (async () => {
      try {
        ollamaHost = await syncOllamaHostFromStorage();
      } catch {
        // Keep stored host; refreshModels will surface errors.
      }
      await refreshModels();
    })();
  });

  $effect(() => {
    if (conversationsHydrated) return;
    conversationsHydrated = true;

    if (conversations.length === 0) {
      activeConversationId = crypto.randomUUID();
      return;
    }

    const sorted = [...conversations].sort((a, b) => b.updatedAt - a.updatedAt);
    const preferredId = appPreferences.lastActiveConversationId;
    const target =
      (preferredId
        ? conversations.find((c) => c.id === preferredId)
        : undefined) ?? sorted[0];

    activeConversationId = target.id;
    messages = target.messages;
    history = target.history;
    agentPlan = target.agentPlan ?? null;
    setSelectedModel(target.model);
    thinkEnabled = target.thinkEnabled;
    chatMode = target.chatMode;
  });

  $effect(() => {
    if (!agentSettings.enabled || agentSettings.allowedRoots.length > 0) return;
    ensureAgentRoots(agentSettings).then((next) => {
      agentSettings = next;
      saveAgentSettings(next);
    });
  });

  $effect(() => {
    if (currentView !== "chat" || messages.length === 0) return;
    const targetId = scrollToMessageId;
    if (targetId) {
      void tick().then(() => {
        document
          .getElementById(`msg-${targetId}`)
          ?.scrollIntoView({ behavior: "smooth", block: "center" });
        scrollToMessageId = null;
      });
      return;
    }
    messagesEnd?.scrollIntoView({ behavior: "smooth" });
  });

  $effect(() => {
    void refreshKokoroDetect();
  });

  $effect(() => {
    return subscribeSpeaking((active) => {
      ttsBusy = active;
      if (!active) speakingTtsTarget = null;
    });
  });

  function onGlobalKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key === "n") {
      event.preventDefault();
      startNewChat();
    }
  }

  function readFileAsBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result;
        if (typeof result !== "string") {
          reject(new Error("Failed to read image"));
          return;
        }
        resolve(result.split(",")[1] ?? result);
      };
      reader.onerror = () =>
        reject(reader.error ?? new Error("Failed to read image"));
      reader.readAsDataURL(file);
    });
  }

  async function handleImageFiles(files: FileList) {
    for (const file of Array.from(files)) {
      const base64 = await readFileAsBase64(file);
      pending = [
        ...pending,
        {
          kind: "image",
          base64,
          label: file.name,
          previewUrl: URL.createObjectURL(file),
        },
      ];
    }
  }

  async function handleAudioFiles(files: FileList) {
    isPreparingAudio = true;
    errorMessage = "";
    try {
      for (const file of Array.from(files)) {
        const result = await prepareAudioForOllama(file);
        pending = [
          ...pending,
          {
            kind: "audio",
            base64: result.base64,
            label: file.name,
            previewUrl: wavBase64ToUrl(result.base64),
            note: formatAudioNote(result),
            durationSecs: result.duration_secs,
          },
        ];
      }
    } catch (error) {
      errorMessage = String(error);
    } finally {
      isPreparingAudio = false;
    }
  }

  function attachmentsForOllama(items: PendingAttachment[]): string[] {
    return [...items]
      .sort((a, b) => (a.kind === "audio" ? -1 : b.kind === "audio" ? 1 : 0))
      .map((item) => item.base64);
  }

  function attachmentsForDisplay(
    items: PendingAttachment[],
  ): AttachmentDisplay[] {
    return items.map((item) => ({
      kind: item.kind,
      label: item.label,
      previewUrl: item.previewUrl,
      note: item.note,
      durationSecs: item.durationSecs,
    }));
  }

  function updateAgentSettings(next: AgentSettings) {
    agentSettings = next;
    saveAgentSettings(next);
  }

  async function pickWorkspaceFolder(): Promise<boolean> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Choose workspace folder for agent tools",
      });
      if (typeof selected !== "string" || !selected.trim()) return false;
      updateAgentSettings({
        ...agentSettings,
        allowedRoots: [
          ...new Set([...agentSettings.allowedRoots, selected.trim()]),
        ],
      });
      showWorkspacePrompt = false;
      errorMessage = "";
      return true;
    } catch (error) {
      errorMessage = String(error);
      return false;
    }
  }

  async function ensureWorkspaceRoots(): Promise<boolean> {
    if (agentSettings.allowedRoots.length > 0) return true;
    showWorkspacePrompt = true;
    return false;
  }

  async function addAgentRoot() {
    await pickWorkspaceFolder();
  }

  function removeAgentRoot(index: number) {
    updateAgentSettings({
      ...agentSettings,
      allowedRoots: agentSettings.allowedRoots.filter((_, i) => i !== index),
    });
  }

  function requestApproval(tool: ToolCallDisplay): Promise<boolean> {
    return new Promise((resolve) => {
      pendingApproval = { tool, resolve };
    });
  }

  function handleApproval(approved: boolean, remember = false) {
    if (!pendingApproval) return;
    const { tool, resolve } = pendingApproval;
    if (approved) {
      if (remember) rememberApproval(tool);
      else if (agentSettings.approvalPolicy === "session") {
        recordSessionApproval(tool);
      }
    }
    resolve(approved);
    pendingApproval = null;
  }

  function upsertSystemPrompt(
    chatHistory: ChatMessage[],
    prompt: string,
  ): ChatMessage[] {
    const idx = chatHistory.findIndex((m) => m.role === "system");
    if (idx >= 0) {
      return chatHistory.map((m, i) =>
        i === idx ? { ...m, content: prompt } : m,
      );
    }
    return [{ role: "system", content: prompt }, ...chatHistory];
  }

  function upsertAgentSystemPrompt(
    chatHistory: ChatMessage[],
    options?: { visionInput?: boolean },
  ): ChatMessage[] {
    if (!agentSettings.enabled) {
      if (options?.visionInput && supportsVision) {
        return upsertSystemPrompt(chatHistory, VISION_AGENT_SUPPLEMENT);
      }
      return chatHistory;
    }
    let prompt = buildAgentSystemPrompt(agentSettings);
    if (options?.visionInput && supportsVision) {
      prompt = `${prompt}\n\n${VISION_AGENT_SUPPLEMENT}`;
    }
    return upsertSystemPrompt(chatHistory, prompt);
  }

  function upsertPlanningSystemPrompt(
    chatHistory: ChatMessage[],
  ): ChatMessage[] {
    return upsertSystemPrompt(
      chatHistory,
      buildPlanningSystemPrompt(agentSettings),
    );
  }

  function permissionDeniedMessage(): string {
    const level = levelLabel(agentSettings.level);
    if (agentSettings.level === "read") {
      return (
        `I can't create or modify files because agent access is **${level}**. ` +
        "Enable **Read & write** or **Full access** in Settings → Agent, then try again."
      );
    }
    return (
      `That operation isn't allowed at the current permission level (**${level}**). ` +
      "Check Settings → Agent to raise access if needed."
    );
  }

  function requestPlanApproval(plan: AgentPlan): Promise<boolean> {
    return new Promise((resolve) => {
      pendingPlanApproval = { plan, resolve };
    });
  }

  function handlePlanApproval(approved: boolean) {
    if (!pendingPlanApproval) return;
    const { plan, resolve } = pendingPlanApproval;
    if (approved && activeConversationId) {
      approvePlanExecution(activeConversationId, plan.id);
    }
    resolve(approved);
    pendingPlanApproval = null;
  }

  async function runToolBatch(
    toolCalls: ToolCallDisplay[],
    options?: { planApproved?: boolean },
  ): Promise<ToolCallDisplay[]> {
    const executed: ToolCallDisplay[] = [];
    for (const tool of toolCalls) {
      const tagged: ToolCallDisplay = { ...tool, source: "agent" };
      if (!isAgentTool(tool.name)) {
        executed.push({
          ...tagged,
          status: "failed",
          result: `Tool "${tool.name}" is not a built-in agent tool.`,
        });
        continue;
      }
      if (
        !options?.planApproved &&
        needsApproval(tagged, agentSettings.approvalPolicy)
      ) {
        const ok = await requestApproval(tagged);
        if (!ok) {
          executed.push({
            ...tagged,
            status: "failed",
            result: "Operation denied by user.",
          });
          continue;
        }
      }
      try {
        const result = await executeAgentTool(tagged, agentSettings);
        const denied = isPermissionDeniedResult(result);
        executed.push({
          ...tagged,
          status: denied ? "failed" : "completed",
          result,
        });
      } catch (error) {
        executed.push({
          ...tagged,
          status: "failed",
          result: String(error),
        });
      }
    }
    return executed;
  }

  async function streamAssistantRound(
    convId: string,
    chatHistory: ChatMessage[],
    assistantId: string,
    sessionId: string,
    useAudioContext: boolean,
    streamModel: string,
    streamThink: boolean | undefined,
    streamTools: boolean,
    toolsOverride?: unknown[],
  ): Promise<{
    thinking: string;
    content: string;
    toolCalls: ToolCallDisplay[];
    cancelled: boolean;
  }> {
    let thinking = "";
    let content = "";
    let toolCalls: ToolCallDisplay[] = [];
    let cancelled = false;
    let thinkingStartedAt: number | null = null;

    const tools =
      toolsOverride ??
      (streamTools && agentSettings.enabled
        ? toolsForLevel(agentSettings)
        : undefined);

    activeSessions.set(convId, sessionId);

    const streamResult = await streamChat(
      sessionId,
      {
        model: streamModel,
        messages: chatHistory,
        think: streamThink,
        tools,
        stream: true,
        options: {
          num_predict: -1,
          num_ctx: 8192,
        },
      },
      (chunk) => {
        if (chunk.cancelled) cancelled = true;
        if (convId === activeConversationId && chunk.error) {
          errorMessage = chunk.error;
        }
        if (chunk.thinking) {
          if (thinkingStartedAt === null) thinkingStartedAt = Date.now();
          thinking += chunk.thinking;
        }
        if (chunk.content) content += chunk.content;
        if (chunk.tool_calls?.length) {
          toolCalls = parseToolCalls(chunk.tool_calls).map((t) => ({
            ...t,
            status: "running" as const,
            source: "agent" as const,
          }));
        }

        patchConvMessage(convId, assistantId, {
          thinking,
          content,
          toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
          streaming: !chunk.done,
        });
      },
    );

    cancelled = cancelled || streamResult.cancelled || abortedConversations.has(convId);
    if (thinkingStartedAt) {
      patchConvMessage(convId, assistantId, {
        thinkingDurationMs: Date.now() - thinkingStartedAt,
      });
    }
    activeSessions.delete(convId);
    return { thinking, content, toolCalls, cancelled };
  }

  async function runAgentLoop(opts: {
    convId: string;
    chatHistory: ChatMessage[];
    initialAssistantId: string;
    streamModel: string;
    streamThink: boolean | undefined;
    streamTools: boolean;
    toolsOverride?: unknown[];
    maxRounds: number;
    planApproved?: boolean;
    includesAudio?: boolean;
    /** Keep going when the model replies without tools (plan execution). */
    persistWithoutTools?: boolean;
  }): Promise<{
    chatHistory: ChatMessage[];
    finished: boolean;
    lastRoundToolFailed: boolean;
  }> {
    const {
      convId,
      initialAssistantId,
      streamModel,
      streamThink,
      streamTools,
      toolsOverride,
      maxRounds,
      planApproved,
      includesAudio = false,
      persistWithoutTools = false,
    } = opts;
    let chatHistory = opts.chatHistory;
    let currentAssistantId = initialAssistantId;
    let finished = false;
    let lastRoundToolFailed = false;

    for (let round = 0; round < maxRounds; round++) {
      if (abortedConversations.has(convId)) break;

      const sessionId = crypto.randomUUID();
      const { thinking, content, toolCalls, cancelled } =
        await streamAssistantRound(
          convId,
          chatHistory,
          currentAssistantId,
          sessionId,
          includesAudio && round === 0,
          streamModel,
          streamThink,
          streamTools,
          toolsOverride,
        );

      if (cancelled || abortedConversations.has(convId)) {
        patchConvMessage(convId, currentAssistantId, {
          thinking,
          content,
          streaming: false,
        });
        break;
      }

      if (toolCalls.length === 0) {
        patchConvMessage(convId, currentAssistantId, {
          thinking,
          content,
          streaming: false,
        });
        chatHistory = [
          ...chatHistory,
          {
            role: "assistant",
            content,
            thinking: thinking || undefined,
          },
        ];

        if (persistWithoutTools && isPlanCompleteMarker(content)) {
          finished = true;
          break;
        }

        if (persistWithoutTools && round < maxRounds - 1) {
          chatHistory.push({
            role: "system",
            content: planContinueNudge(),
          });
          currentAssistantId = crypto.randomUUID();
          appendConvMessage(convId, {
            id: currentAssistantId,
            role: "assistant",
            content: "",
            thinking: "",
            streaming: true,
          });
          continue;
        }

        finished = !persistWithoutTools;
        break;
      }

      if (abortedConversations.has(convId)) break;

      if (persistWithoutTools) {
        patchPlanStep(convId, round, "running");
      }
      const executed = await runToolBatch(toolCalls, { planApproved });
      if (abortedConversations.has(convId)) break;

      const denied = executed.some((t) => isPermissionDeniedResult(t.result));
      const toolFailed = executed.some((t) => t.status === "failed");
      lastRoundToolFailed = denied || toolFailed;
      if (persistWithoutTools) {
        patchPlanStep(
          convId,
          round,
          lastRoundToolFailed ? "failed" : "done",
        );
      }
      patchConvMessage(convId, currentAssistantId, {
        thinking,
        content: content || "",
        toolCalls: executed,
        streaming: false,
      });

      chatHistory = [
        ...chatHistory,
        {
          role: "assistant",
          content: "",
          tool_calls: toolCallsForOllamaHistory(toolCalls),
        },
      ];

      for (const tool of executed) {
        chatHistory.push({
          role: "tool",
          content: tool.result ?? "Error",
          tool_name: tool.name,
        });
      }

      if (denied) {
        currentAssistantId = crypto.randomUUID();
        appendConvMessage(convId, {
          id: currentAssistantId,
          role: "assistant",
          content: "",
          thinking: "",
          streaming: true,
        });
        chatHistory.push({
          role: "system",
          content:
            "A tool was denied due to permission level. Respond to the user explaining what access level is required. Do not call any more tools.",
        });
        const finalSessionId = crypto.randomUUID();
        const final = await streamAssistantRound(
          convId,
          chatHistory,
          currentAssistantId,
          finalSessionId,
          false,
          streamModel,
          streamThink,
          false,
        );
        patchConvMessage(convId, currentAssistantId, {
          thinking: final.thinking,
          content: final.content.trim() || permissionDeniedMessage(),
          streaming: false,
        });
        chatHistory = [
          ...chatHistory,
          {
            role: "assistant",
            content: final.content.trim() || permissionDeniedMessage(),
            thinking: final.thinking || undefined,
          },
        ];
        finished = true;
        break;
      }

      currentAssistantId = crypto.randomUUID();
      appendConvMessage(convId, {
        id: currentAssistantId,
        role: "assistant",
        content: "",
        thinking: "",
        toolCalls: [],
        streaming: true,
      });
    }

    return { chatHistory, finished, lastRoundToolFailed };
  }

  function getActivePlan(convId: string): AgentPlan | null {
    if (convId === activeConversationId) return agentPlan;
    return conversations.find((c) => c.id === convId)?.agentPlan ?? null;
  }

  async function generatePlan(
    convId: string,
    rawContent: string,
    attachments: PendingAttachment[],
    autoTriggered = false,
  ) {
    if (!(await ensureWorkspaceRoots())) return;

    messages = [
      ...messages,
      {
        id: crypto.randomUUID(),
        role: "user",
        content: rawContent,
        attachments: attachmentsForDisplay(attachments),
      },
    ];

    history = [
      ...history,
      {
        role: "user",
        content: buildPlanningUserMessage(
          rawContent,
          agentSettings.allowedRoots,
        ),
        images:
          attachments.length > 0 ? attachmentsForOllama(attachments) : undefined,
      },
    ];

    const assistantId = crypto.randomUUID();
    messages = [
      ...messages,
      {
        id: assistantId,
        role: "assistant",
        content: "",
        thinking: "",
        streaming: true,
      },
    ];

    addStreaming(convId);
    errorMessage = "";

    const streamModel = selectedModel;
    const streamThink = supportsThinking ? thinkEnabled : undefined;

    let chatHistory = upsertPlanningSystemPrompt([...history]);
    let parsedPlan: AgentPlan | null = null;

    try {
      ({ chatHistory } = await runAgentLoop({
        convId,
        chatHistory,
        initialAssistantId: assistantId,
        streamModel,
        streamThink,
        streamTools: true,
        toolsOverride: toolsForPlanning(agentSettings),
        maxRounds: PLAN_MAX_ROUNDS,
      }));

      const lastAssistant = [...getConversationMessages(convId)]
        .reverse()
        .find((m) => m.role === "assistant" && !m.streaming);
      const rawPlanContent = [
        lastAssistant?.content ?? "",
        lastAssistant?.thinking ?? "",
      ].join("\n");
      parsedPlan = parsePlanFromContent(rawPlanContent, rawContent);

      if (parsedPlan) {
        setConversationPlan(convId, parsedPlan);
        const summaryId = lastAssistant?.id ?? assistantId;
        const summary = autoTriggered
          ? `${autoPlanNotice()}\n\n${planReadySummary()}`
          : planReadySummary();
        patchConvMessage(convId, summaryId, {
          content: summary,
          thinking: lastAssistant?.thinking,
          toolCalls: lastAssistant?.toolCalls,
        });
      } else if (lastAssistant) {
        patchConvMessage(convId, lastAssistant.id, {
          content:
            (lastAssistant.content || "").trim() ||
            "I couldn't build a structured plan. Try rephrasing your request.",
        });
      }

      setConversationHistory(convId, chatHistory);
      syncActiveConversation(true);
    } catch (error) {
      if (convId === activeConversationId) errorMessage = String(error);
      patchConvMessage(convId, assistantId, {
        content: "Failed to generate a plan.",
        streaming: false,
      });
      syncActiveConversation(true);
    } finally {
      activeSessions.delete(convId);
      abortedConversations.delete(convId);
      removeStreaming(convId);
    }

  }

  async function executePlan() {
    if (!activeConversationId || !agentPlan || isActiveStreaming) return;
    if (!supportsTools) {
      notifyToolsUnavailable("run a plan");
      return;
    }
    if (!(await ensureWorkspaceRoots())) return;

    const convId = activeConversationId;
    let plan = agentPlan;

    if (plan.status === "failed") {
      plan = {
        ...plan,
        status: "ready",
        steps: plan.steps.map((s) =>
          s.status === "failed" ? { ...s, status: "pending" as PlanStepStatus } : s,
        ),
      };
      setConversationPlan(convId, plan);
    }

    if (planWriteSteps(plan).length > 0) {
      const alreadyApproved = isPlanExecutionApproved(convId, plan.id);
      if (!alreadyApproved) {
        const ok = await requestPlanApproval(plan);
        if (!ok) return;
      }
    }

    const executingPlan: AgentPlan = {
      ...plan,
      status: "executing",
      steps: plan.steps.map((s) => ({
        ...s,
        status: s.status === "done" ? ("done" as PlanStepStatus) : ("pending" as PlanStepStatus),
      })),
    };
    setConversationPlan(convId, executingPlan);

    addStreaming(convId);
    errorMessage = "";

    const streamModel = selectedModel;
    const streamThink = supportsThinking ? thinkEnabled : undefined;
    let chatHistory = upsertAgentSystemPrompt([...history]);

    let allStepsOk = true;

    try {
      for (let i = 0; i < executingPlan.steps.length; i++) {
        if (abortedConversations.has(convId)) {
          allStepsOk = false;
          break;
        }

        const livePlan = getActivePlan(convId) ?? executingPlan;
        const step = livePlan.steps[i];
        if (!step || step.status === "done") continue;

        patchPlanStep(convId, i, "running");

        const assistantId = crypto.randomUUID();
        appendConvMessage(convId, {
          id: assistantId,
          role: "assistant",
          content: "",
          thinking: "",
          streaming: true,
        });

        const stepHistory: ChatMessage[] = [
          ...chatHistory,
          {
            role: "system",
            content: buildSingleStepExecutionPrompt(
              agentSettings,
              livePlan,
              i,
            ),
          },
          { role: "user", content: buildSingleStepUserMessage(step) },
        ];

        const { chatHistory: afterStep, lastRoundToolFailed, finished } =
          await runAgentLoop({
            convId,
            chatHistory: stepHistory,
            initialAssistantId: assistantId,
            streamModel,
            streamThink,
            streamTools: supportsTools,
            maxRounds: STEP_MAX_ROUNDS,
            planApproved: true,
          });

        chatHistory = afterStep;

        const lastBubble = getConversationMessages(convId).find(
          (m) => m.id === assistantId,
        );
        const stepContent = lastBubble?.content ?? "";
        const stepOk =
          !lastRoundToolFailed &&
          !abortedConversations.has(convId) &&
          (finished || isPlanCompleteMarker(stepContent) || !lastRoundToolFailed);

        patchPlanStep(convId, i, stepOk ? "done" : "failed");

        if (lastBubble && stepContent.trim()) {
          chatHistory.push({
            role: "assistant",
            content: stepContent,
            thinking: lastBubble.thinking,
          });
        }

        if (!stepOk) {
          allStepsOk = false;
          break;
        }
      }

      const finalPlan = getActivePlan(convId) ?? executingPlan;
      const completed = allStepsOk && allPlanStepsDone(finalPlan);

      setConversationPlan(convId, {
        ...finalPlan,
        status: completed ? "completed" : "failed",
      });
      setConversationHistory(convId, chatHistory);
      syncActiveConversation(true);
    } catch (error) {
      if (convId === activeConversationId) errorMessage = String(error);
      const failedPlan = getActivePlan(convId) ?? executingPlan;
      setConversationPlan(convId, { ...failedPlan, status: "failed" });
      syncActiveConversation(true);
    } finally {
      activeSessions.delete(convId);
      abortedConversations.delete(convId);
      removeStreaming(convId);
    }
  }

  function discardPlan() {
    if (!activeConversationId) return;
    setConversationPlan(activeConversationId, null);
  }

  function updatePlan(next: AgentPlan) {
    if (!activeConversationId) return;
    setConversationPlan(activeConversationId, next);
  }

  async function sendMessage() {
    const trimmed = input.trim();
    if (
      (!trimmed && pending.length === 0) ||
      !selectedModel ||
      isActiveStreaming ||
      isPreparingAudio
    ) {
      return;
    }

    if (!activeConversationId) {
      activeConversationId = crypto.randomUUID();
    }

    const convId = activeConversationId;

    if (streamingIds.size >= appPreferences.maxConcurrentChats) {
      errorMessage = `Maximum ${appPreferences.maxConcurrentChats} concurrent chats. Wait for one to finish or raise the limit in Settings.`;
      return;
    }

    const includesAudio = hasPendingAudio;
    const rawContent =
      trimmed || (includesAudio ? DEFAULT_AUDIO_PROMPT : "");
    const savedPending = [...pending];
    const savedMode = chatMode;

    input = "";
    pending = [];
    chatMode = "ask";

    const hasVisionInput = savedPending.some((a) => a.kind === "image");

    const autoPlan =
      savedMode === "ask" &&
      agentSettings.enabled &&
      supportsTools &&
      !hasVisionInput &&
      shouldAutoPlan(rawContent);

    const willUseAgentTools =
      agentSettings.enabled && supportsTools && !hasVisionInput;

    if (willUseAgentTools && !(await ensureWorkspaceRoots())) {
      input = rawContent;
      pending = savedPending;
      chatMode = savedMode;
      return;
    }

    if (savedMode === "plan" || autoPlan) {
      if (!agentSettings.enabled || !supportsTools) {
        notifyToolsUnavailable("use Plan mode");
        input = rawContent;
        pending = savedPending;
        chatMode = savedMode;
        return;
      }
      await generatePlan(convId, rawContent, savedPending, autoPlan);
      return;
    }

    if (agentSettings.enabled && !supportsTools && !hasVisionInput) {
      notifyToolsUnavailable("run agent tools");
    }

    const messageContent = applyModePrefix(rawContent, savedMode);

    messages = [
      ...messages,
      {
        id: crypto.randomUUID(),
        role: "user",
        content: rawContent,
        attachments: attachmentsForDisplay(savedPending),
      },
    ];

    history = [
      ...history,
      {
        role: "user",
        content: messageContent,
        images:
          savedPending.length > 0
            ? attachmentsForOllama(savedPending)
            : undefined,
      },
    ];

    const assistantId = crypto.randomUUID();
    messages = [
      ...messages,
      {
        id: assistantId,
        role: "assistant",
        content: "",
        thinking: "",
        toolCalls: [],
        streaming: true,
      },
    ];

    addStreaming(convId);
    errorMessage = "";

    const streamModel = selectedModel;
    const streamThink = supportsThinking ? thinkEnabled : undefined;
    // Vision requests: image is in the message — don't offer agent tools.
    const streamTools =
      supportsTools && agentSettings.enabled && !hasVisionInput;

    let chatHistory = upsertAgentSystemPrompt([...history], {
      visionInput: hasVisionInput,
    });

    try {
      ({ chatHistory } = await runAgentLoop({
        convId,
        chatHistory,
        initialAssistantId: assistantId,
        streamModel,
        streamThink,
        streamTools,
        maxRounds: MAX_TOOL_ROUNDS,
        includesAudio,
      }));

      setConversationHistory(convId, chatHistory);
      syncActiveConversation(true);

      const finalReply = getConversationMessages(convId).find(
        (m) => m.id === assistantId,
      );
      if (finalReply?.content?.trim()) {
        void maybeAutoSpeak(finalReply.content);
      }
    } catch (error) {
      if (convId === activeConversationId) errorMessage = String(error);
      patchConvMessage(convId, assistantId, {
        content: "Failed to get a response.",
        streaming: false,
      });
      syncActiveConversation(true);
    } finally {
      activeSessions.delete(convId);
      abortedConversations.delete(convId);
      removeStreaming(convId);
    }
  }
</script>

<svelte:window onkeydown={onGlobalKeydown} />

<div class="shell" class:sidebar-collapsed={appPreferences.sidebarCollapsed}>
  <AppSidebar
    {conversations}
    {activeConversationId}
    {currentView}
    {connected}
    {statusMessage}
    {chatsExpanded}
    collapsed={appPreferences.sidebarCollapsed}
    streamingIds={streamingIdList}
    onNewChat={startNewChat}
    onSelectConversation={selectConversation}
    onDeleteConversation={deleteConversation}
    onNavigate={navigate}
    onToggleChats={() => (chatsExpanded = !chatsExpanded)}
    onToggleCollapse={toggleSidebarCollapsed}
    taggedCount={taggedMessages.length}
  />

  <div class="main">
    {#if currentView === "chat"}
      <header class="chat-header">
        <h2>{chatTitle}</h2>
        {#if messages.some((m) => m.content.trim() && !m.streaming)}
          <button
            type="button"
            class="header-export-btn"
            onclick={exportChatMarkdown}
            title="Download chat as Markdown"
            aria-label="Download chat as Markdown"
          >
            ↓ Export .md
          </button>
        {/if}
        {#if selectedModelInfo}
          <div class="header-badges">
            {#each selectedModelInfo.capabilities as cap}
              <span class="badge">{cap}</span>
            {/each}
          </div>
        {/if}
      </header>

      <div class="thread-scroll">
        <div class="thread">
          {#if messages.length === 0}
            <div class="empty">
              <h2>Chat with local models</h2>
              <p>
                Use <strong>+</strong> → <strong>Plan</strong> to explore read-only,
                then run the plan from the Plan panel. Enable agent access in Settings
                for file tools.
              </p>
            </div>
          {:else}
            {#each messages as message, index (message.id)}
              <div
                id="msg-{message.id}"
                class="row"
                class:user={message.role === "user"}
                class:assistant={message.role === "assistant"}
              >
                <ChatBubble
                  role={message.role}
                  messageId={message.id}
                  content={message.content}
                  thinking={message.thinking}
                  thinkingDurationMs={message.thinkingDurationMs}
                  attachments={message.attachments}
                  toolCalls={message.toolCalls}
                  contextToolCalls={contextToolCallsForMessage(index)}
                  streaming={message.streaming}
                  showThinking={thinkEnabled}
                  agentLayout={agentSettings.enabled && message.role === "assistant"}
                  ttsEnabled={ttsSettings.enabled && kokoroDetected}
                  speaking={speakingTtsTarget === message.id}
                  speakingBlockKey={speakingBlockKeyFor(message.id)}
                  onSpeak={
                    message.role === "assistant" && message.content.trim()
                      ? () => speakMessage(message.id, message.content)
                      : undefined
                  }
                  onSpeakBlock={
                    message.role === "assistant"
                      ? (text, blockKey) =>
                          speakMessageBlock(message.id, blockKey, text)
                      : undefined
                  }
                  onStopSpeak={stopTts}
                  allowBlockDownload={
                    message.role === "assistant" &&
                    Boolean(message.content.trim()) &&
                    !message.streaming
                  }
                  tagged={
                    activeConversationId
                      ? isMessageTagged(
                          taggedMessages,
                          activeConversationId,
                          message.id,
                        )
                      : false
                  }
                  onCopy={
                    message.content.trim() && !message.streaming
                      ? () => void copyToClipboard(message.content)
                      : undefined
                  }
                  onBranch={
                    !message.streaming && !isActiveStreaming
                      ? () => branchFromMessage(index)
                      : undefined
                  }
                  onToggleTag={
                    message.content.trim() && !message.streaming
                      ? () => toggleMessageTag(message)
                      : undefined
                  }
                />
              </div>
            {/each}
            <div bind:this={messagesEnd}></div>
          {/if}
        </div>
      </div>

      {#if errorMessage}
        <div class="error-banner" role="alert">{errorMessage}</div>
      {/if}

      <PlanPanel
        plan={agentPlan}
        isStreaming={isActiveStreaming}
        onRun={executePlan}
        onDiscard={discardPlan}
        onPlanChange={updatePlan}
      />

      <ChatComposer
        {input}
        {pending}
        {models}
        {selectedModel}
        {chatMode}
        {connected}
        isStreaming={isActiveStreaming}
        {isPreparingAudio}
        {supportsVision}
        supportsAudio={supportsAudioInput}
        {supportsTools}
        {hasPendingAudio}
        onInputChange={(v) => (input = v)}
        onSend={sendMessage}
        onRemovePending={(i) => (pending = pending.filter((_, idx) => idx !== i))}
        onImageFiles={handleImageFiles}
        onAudioFiles={handleAudioFiles}
        onModelChange={setSelectedModel}
        onModeChange={(m) => {
          if (
            m === "plan" &&
            (!agentSettings.enabled || !supportsTools)
          ) {
            notifyToolsUnavailable("use Plan mode");
          }
          chatMode = m;
        }}
        onOpenModelsView={() => navigate("models")}
        onOpenSettingsView={() => navigate("settings")}
        onStop={stopStreaming}
      />
    {:else if currentView === "tagged"}
      <div class="view-scroll">
        <TaggedView
          tags={taggedMessages}
          onOpen={openTaggedMessage}
          onRemove={removeTaggedMessageById}
        />
      </div>
    {:else if currentView === "models"}
      <div class="view-scroll">
        <ModelsView
          {models}
          {selectedModel}
          {connected}
          onSelectModel={setSelectedModel}
          onRefresh={refreshModels}
          onStartChat={() => navigate("chat")}
        />
      </div>
    {:else if currentView === "settings"}
      <div class="view-scroll">
        <SettingsView
          {thinkEnabled}
          {supportsThinking}
          supportsVision={supportsVision}
          supportsAudio={supportsAudioInput}
          {supportsTools}
          {connected}
          {ollamaHost}
          isStreaming={streamingIds.size > 0}
          {agentSettings}
          {selectedModel}
          toolSuggestions={toolModelSuggestions}
          thinkingSuggestions={thinkingModelSuggestions}
          maxConcurrentChats={appPreferences.maxConcurrentChats}
          onThinkChange={(v) => (thinkEnabled = v)}
          onRefresh={refreshModels}
          onOllamaHostChange={(value) => (ollamaHost = value)}
          onAgentChange={updateAgentSettings}
          onAddRoot={addAgentRoot}
          onRemoveRoot={removeAgentRoot}
          onBrowseModels={() => navigate("models")}
          onSelectModel={setSelectedModel}
          onMaxConcurrentChange={(n) => updateAppPreferences({ maxConcurrentChats: n })}
          {ttsSettings}
          {kokoroDetected}
          {kokoroPath}
          {ttsBusy}
          onTtsChange={updateTtsSettings}
          onDetectKokoro={refreshKokoroDetect}
          onTestTts={testTtsVoice}
          onStopTts={stopTts}
        />
      </div>
    {/if}
  </div>
</div>

<ToolApprovalModal
  tool={pendingApproval?.tool ?? null}
  onApprove={(remember) => handleApproval(true, remember)}
  onDeny={() => handleApproval(false)}
/>

<PlanApprovalModal
  plan={pendingPlanApproval?.plan ?? null}
  onApprove={() => handlePlanApproval(true)}
  onDeny={() => handlePlanApproval(false)}
/>

<WorkspacePromptModal
  open={showWorkspacePrompt}
  onChooseFolder={() => {
    void pickWorkspaceFolder();
  }}
  onDismiss={() => (showWorkspacePrompt = false)}
/>

<OllamaSetupModal
  open={showOllamaSetup}
  host={ollamaHost}
  error={errorMessage}
  busy={ollamaCheckBusy}
  onHostChange={(value) => (ollamaHost = value)}
  onRetry={retryOllamaSetup}
  onOpenSettings={openOllamaSettings}
  onDismiss={dismissOllamaSetup}
/>

<ToastStack />

<style>
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--color-bg);
    transition: grid-template-columns 0.15s ease;
  }

  .shell.sidebar-collapsed {
    grid-template-columns: var(--sidebar-width-collapsed) 1fr;
  }

  .main {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    height: 100%;
    overflow: hidden;
    background: var(--color-bg);
  }

  .view-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .chat-header {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-4) var(--space-2);
    border-bottom: 1px solid var(--color-border-subtle);
    max-width: calc(var(--thread-max-width) + var(--space-8) * 2);
    margin: 0 auto;
    width: 100%;
  }

  .chat-header h2 {
    margin: 0;
    font-size: var(--text-base);
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .header-export-btn {
    flex-shrink: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-inset);
    color: var(--color-text-secondary);
    font-size: var(--text-xs);
    padding: 4px 10px;
    cursor: pointer;
  }

  .header-export-btn:hover {
    color: var(--color-text);
    border-color: var(--color-primary);
  }

  .header-badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    flex-shrink: 0;
  }

  .badge {
    font-size: 0.65rem;
    padding: 2px 6px;
    border-radius: var(--radius-full);
    background: rgba(79, 106, 245, 0.12);
    color: #9ec5ff;
  }

  .thread-scroll {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  .thread {
    max-width: calc(var(--thread-max-width) + var(--space-8) * 2);
    margin: 0 auto;
    padding: var(--space-6) var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    width: 100%;
    min-height: 100%;
  }

  .thread :global(.alert) {
    margin-bottom: var(--space-2);
  }

  .row {
    display: flex;
    width: 100%;
  }

  .row.user { justify-content: flex-end; }
  .row.assistant { justify-content: flex-start; }

  .row.user :global(.bubble) {
    max-width: min(100%, 560px);
  }

  .empty {
    margin: auto;
    padding: var(--space-8) var(--space-4);
    text-align: center;
    max-width: 420px;
  }

  .empty h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-lg);
  }

  .empty p {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  .error-banner {
    flex-shrink: 0;
    margin: 0 var(--space-4) var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--color-error-bg);
    border: 1px solid rgba(248, 113, 113, 0.35);
    color: var(--color-error);
    font-size: var(--text-sm);
    max-width: calc(var(--thread-max-width) + var(--space-8) * 2);
    margin-left: auto;
    margin-right: auto;
    width: calc(100% - var(--space-8));
  }
</style>
