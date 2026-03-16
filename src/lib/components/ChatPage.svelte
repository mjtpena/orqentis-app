<script lang="ts">
  import { chatSessions } from '../stores/data';
  import { createSession, updateSessionLastMessage, deleteSession, loadSessionMessages, persistMessage, persistMessageUpdate } from '../stores/data';
  import { activeEndpoint, authStatus, armDeployments, localAgents } from '../stores/auth';
  import * as api from '../services/api';
  import { notify } from '../services/notifications';
  import type { ChatMessagePayload, FoundryAgent, LocalAgent } from '../services/api';
  import type { ChatMessage, ChatSession } from '../types';
  import { listen } from '@tauri-apps/api/event';
  import MarkdownRenderer from './MarkdownRenderer.svelte';

  // ------------------------------------------------------------------
  // Chat target: what we're talking to
  // ------------------------------------------------------------------
  type ChatTargetKind = 'deployment' | 'agent' | 'local';
  interface ChatTarget {
    kind: ChatTargetKind;
    id: string;         // deployment name, agent id, or local agent id
    name: string;       // display name
    model: string;      // model name
    detail: string;     // subtitle info
    icon: string;       // emoji
    endpoint?: string;  // for local agents
    threadId?: string;  // for Foundry agent thread persistence
  }

  let messages = $state<ChatMessage[]>([]);
  let inputText = $state('');
  let isStreaming = $state(false);
  let chatMessagesEl: HTMLDivElement;
  let textareaEl: HTMLTextAreaElement;

  // Session management
  let activeSessionId = $state<string | null>(null);

  // Current chat target
  let chatTarget = $state<ChatTarget | null>(null);

  // Live agents from Foundry
  let liveAgents = $state<FoundryAgent[]>([]);
  let deploymentsLoading = $state(false);

  // Use ARM deployments from the store
  let deployments = $derived($armDeployments);

  // Abort controller for stopping generation
  let abortController = $state<AbortController | null>(null);

  $effect(() => {
    const endpoint = $activeEndpoint;
    const status = $authStatus;
    if (status.signed_in && endpoint) {
      deploymentsLoading = true;
      api.listAgents(endpoint)
        .then((agents) => { liveAgents = agents; })
        .catch((e) => { console.error('[ChatPage] Failed to load agents:', e); })
        .finally(() => { deploymentsLoading = false; });
    } else {
      liveAgents = [];
    }
  });

  // Auto-select first deployment when available
  $effect(() => {
    if (deployments.length > 0 && !chatTarget) {
      selectDeployment(deployments[0]);
    }
  });

  function scrollToBottom() {
    if (chatMessagesEl) {
      requestAnimationFrame(() => { chatMessagesEl.scrollTop = chatMessagesEl.scrollHeight; });
    }
  }

  function autoResizeTextarea() {
    if (textareaEl) {
      textareaEl.style.height = 'auto';
      textareaEl.style.height = Math.min(textareaEl.scrollHeight, 160) + 'px';
    }
  }

  // ------------------------------------------------------------------
  // Target selection
  // ------------------------------------------------------------------
  function selectDeployment(d: typeof deployments[0]) {
    chatTarget = {
      kind: 'deployment',
      id: d.name,
      name: d.properties.model?.name ?? d.name,
      model: d.properties.model?.name ?? d.name,
      detail: `Foundry · ${d.sku?.name || 'Deployment'}${d.properties.model?.version ? ` · v${d.properties.model.version}` : ''}`,
      icon: '🧠',
    };
  }

  function selectAgent(agent: FoundryAgent) {
    chatTarget = {
      kind: 'agent',
      id: agent.id,
      name: agent.name || agent.id,
      model: agent.model,
      detail: `Foundry Agent · ${agent.model}`,
      icon: '🤖',
      threadId: undefined,
    };
    // Start fresh conversation for agent
    startNewChat();
  }

  function selectLocalAgent(agent: LocalAgent) {
    chatTarget = {
      kind: 'local',
      id: agent.id,
      name: agent.name,
      model: agent.model ?? agent.name,
      detail: `${agent.runtime} · Local`,
      icon: '🏠',
      endpoint: agent.endpoint,
    };
  }

  // ------------------------------------------------------------------
  // Session management
  // ------------------------------------------------------------------
  async function ensureSession(): Promise<string> {
    if (activeSessionId) return activeSessionId;
    const id = crypto.randomUUID();
    const targetName = chatTarget?.name ?? 'Unknown';
    const targetSource = chatTarget?.kind === 'local' ? 'local' : chatTarget?.kind === 'agent' ? 'foundry' : 'model';
    const session: ChatSession = {
      id,
      title: `Chat with ${targetName}`,
      targetName,
      targetSource,
      targetIcon: chatTarget?.icon ?? '💬',
      updatedAt: new Date().toISOString(),
    };
    await createSession(session);
    activeSessionId = id;
    return id;
  }

  async function selectSession(session: ChatSession) {
    activeSessionId = session.id;
    try {
      const loaded = await loadSessionMessages(session.id);
      messages = loaded.map(m => ({
        id: m.id,
        role: m.role as ChatMessage['role'],
        content: m.content,
        timestamp: m.timestamp,
      }));
      scrollToBottom();
    } catch (e) {
      console.error('[ChatPage] Failed to load session messages:', e);
    }
  }

  function startNewChat() {
    activeSessionId = null;
    messages = [];
    inputText = '';
  }

  // ------------------------------------------------------------------
  // Send message (routes to correct backend based on chatTarget)
  // ------------------------------------------------------------------
  async function send() {
    const text = inputText.trim();
    if (!text || isStreaming) return;

    const sessionId = await ensureSession();
    const userMsg: ChatMessage = { id: crypto.randomUUID(), role: 'user', content: text, timestamp: new Date() };
    messages = [...messages, userMsg];
    inputText = '';
    isStreaming = true;
    if (textareaEl) textareaEl.style.height = 'auto';
    scrollToBottom();

    await persistMessage(sessionId, userMsg);

    const endpoint = $activeEndpoint;

    if (chatTarget?.kind === 'local' && chatTarget.endpoint) {
      // ---- LOCAL MODEL CHAT (OpenAI-compatible streaming) ----
      await sendLocalChat(sessionId, chatTarget.endpoint, chatTarget.model, text);
    } else if (chatTarget?.kind === 'agent' && $authStatus.signed_in && endpoint) {
      // ---- FOUNDRY AGENT CHAT (thread/run based) ----
      await sendAgentChat(sessionId, endpoint, chatTarget.id, text);
    } else if (chatTarget?.kind === 'deployment' && $authStatus.signed_in && endpoint) {
      // ---- FOUNDRY DEPLOYMENT CHAT (streaming SSE) ----
      await sendDeploymentChat(sessionId, endpoint, chatTarget.id);
    } else {
      const reply: ChatMessage = { id: crypto.randomUUID(), role: 'assistant', content: 'Please sign in and select a deployment, agent, or local model to start chatting.', timestamp: new Date() };
      messages = [...messages, reply];
      await persistMessage(sessionId, reply);
      isStreaming = false;
      scrollToBottom();
    }
  }

  // ---- Deployment streaming chat ----
  async function sendDeploymentChat(sessionId: string, endpoint: string, deploymentName: string) {
    const assistantId = crypto.randomUUID();
    messages = [...messages, { id: assistantId, role: 'assistant', content: '', timestamp: new Date() }];
    scrollToBottom();

    const unlisten = await listen<{ content?: string; done?: boolean }>('chat-token', (event) => {
      if (event.payload.content) {
        const idx = messages.findIndex(m => m.id === assistantId);
        if (idx !== -1) {
          messages[idx] = { ...messages[idx], content: messages[idx].content + event.payload.content };
          messages = [...messages];
          scrollToBottom();
        }
      }
      if (event.payload.done) {
        finishStreaming(sessionId, assistantId);
      }
    });

    const payload: ChatMessagePayload[] = messages
      .filter(m => m.role !== 'system')
      .slice(0, -1)
      .map(m => ({ role: m.role, content: m.content }));

    try {
      await api.sendChatMessage(endpoint, deploymentName, payload);
    } catch (e: any) {
      handleStreamError(assistantId, e);
    } finally {
      unlisten();
      if (isStreaming) isStreaming = false;
    }
  }

  // ---- Local model streaming chat ----
  async function sendLocalChat(sessionId: string, endpoint: string, model: string, _text: string) {
    const assistantId = crypto.randomUUID();
    messages = [...messages, { id: assistantId, role: 'assistant', content: '', timestamp: new Date() }];
    scrollToBottom();

    const unlisten = await listen<{ content?: string; done?: boolean }>('chat-token', (event) => {
      if (event.payload.content) {
        const idx = messages.findIndex(m => m.id === assistantId);
        if (idx !== -1) {
          messages[idx] = { ...messages[idx], content: messages[idx].content + event.payload.content };
          messages = [...messages];
          scrollToBottom();
        }
      }
      if (event.payload.done) {
        finishStreaming(sessionId, assistantId);
      }
    });

    const payload: ChatMessagePayload[] = messages
      .filter(m => m.role !== 'system')
      .slice(0, -1)
      .map(m => ({ role: m.role, content: m.content }));

    try {
      await api.sendLocalChatMessage(endpoint, model, payload);
    } catch (e: any) {
      handleStreamError(assistantId, e);
    } finally {
      unlisten();
      if (isStreaming) isStreaming = false;
    }
  }

  // ---- Foundry Agent chat (thread + run) ----
  async function sendAgentChat(sessionId: string, endpoint: string, agentId: string, text: string) {
    const assistantId = crypto.randomUUID();
    messages = [...messages, { id: assistantId, role: 'assistant', content: '', timestamp: new Date() }];
    scrollToBottom();

    try {
      const result = await api.sendAgentMessage(endpoint, agentId, chatTarget?.threadId ?? null, text);

      // Store thread ID for follow-up messages
      if (chatTarget) {
        chatTarget = { ...chatTarget, threadId: result.thread_id };
      }

      // Find the latest assistant message from the run result
      const assistantMessages = result.messages.filter(m => m.role === 'assistant');
      const lastAssistantMsg = assistantMessages[assistantMessages.length - 1];

      if (lastAssistantMsg) {
        const idx = messages.findIndex(m => m.id === assistantId);
        if (idx !== -1) {
          messages[idx] = { ...messages[idx], content: lastAssistantMsg.content };
          messages = [...messages];
        }
      } else {
        const idx = messages.findIndex(m => m.id === assistantId);
        if (idx !== -1) {
          messages[idx] = { ...messages[idx], content: `Agent run completed with status: ${result.status}` };
          messages = [...messages];
        }
      }

      // Persist
      const finalMsg = messages.find(m => m.id === assistantId);
      if (finalMsg) {
        await persistMessage(sessionId, finalMsg);
        const preview = finalMsg.content.slice(0, 80);
        await updateSessionLastMessage(sessionId, preview);
      }
      notify('Agent Response', `${chatTarget?.name ?? 'Agent'} replied.`);
    } catch (e: any) {
      handleStreamError(assistantId, e);
    } finally {
      isStreaming = false;
      scrollToBottom();
    }
  }

  // ---- Shared helpers ----
  function finishStreaming(sessionId: string, assistantId: string) {
    isStreaming = false;
    scrollToBottom();
    const finalMsg = messages.find(m => m.id === assistantId);
    if (finalMsg) {
      persistMessage(sessionId, finalMsg);
      const preview = finalMsg.content.slice(0, 80);
      updateSessionLastMessage(sessionId, preview);
    }
  }

  function handleStreamError(assistantId: string, e: any) {
    const idx = messages.findIndex(m => m.id === assistantId);
    if (idx !== -1) {
      messages[idx] = { ...messages[idx], content: `⚠️ Error: ${e}` };
      messages = [...messages];
    }
    isStreaming = false;
  }

  function stopGeneration() {
    // Note: We can't truly abort Tauri invocations, but we can stop listening
    isStreaming = false;
  }

  async function copyMessage(content: string) {
    await navigator.clipboard.writeText(content);
  }

  async function regenerateMessage(msgIndex: number) {
    if (isStreaming) return;
    // Remove messages from this index onward and resend
    const userMsgBefore = messages.slice(0, msgIndex).filter(m => m.role === 'user').pop();
    if (!userMsgBefore) return;
    messages = messages.slice(0, msgIndex);
    inputText = userMsgBefore.content;
    // Remove the user message too so send() re-adds it
    messages = messages.slice(0, -1);
    await send();
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  }

  async function handleDeleteSession(id: string, e: MouseEvent) {
    e.stopPropagation();
    await deleteSession(id);
    if (activeSessionId === id) {
      startNewChat();
    }
  }

  function handleSuggestion(text: string) {
    inputText = text;
    send();
  }

  // ------------------------------------------------------------------
  // Derived UI state
  // ------------------------------------------------------------------
  let topbarName = $derived(chatTarget?.name ?? 'Select a model');
  let topbarDetail = $derived(chatTarget?.detail ?? 'Choose a deployment, agent, or local model');
  let topbarIcon = $derived(chatTarget?.icon ?? '💬');
  let chatTargetBadge = $derived(
    chatTarget?.kind === 'local' ? 'Local' :
    chatTarget?.kind === 'agent' ? 'Agent' :
    $authStatus.signed_in ? 'Live' : 'Offline'
  );
  let chatTargetBadgeClass = $derived(
    chatTarget?.kind === 'local' ? 'badge-yellow' :
    chatTarget?.kind === 'agent' ? 'badge-purple' :
    $authStatus.signed_in ? 'badge-blue' : 'badge-muted'
  );

  let totalTokens = $derived(
    messages.reduce((sum, m) => sum + Math.ceil(m.content.length / 4), 0)
  );

  // Suggestion chips
  const suggestions = [
    'Explain how transformers work',
    'Write a Python quicksort',
    'Compare Azure OpenAI vs OpenAI',
    'What are AI agents?',
  ];
</script>

<div class="chat-shell">
  <!-- Sidebar -->
  <div class="chat-list">
    <div class="chat-list-header">
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:10px">
        <span style="font-weight:600;font-size:.95rem">Chat</span>
        <button class="btn-icon" style="font-size:.82rem" onclick={startNewChat} title="New chat">+</button>
      </div>
      <div class="search-box" style="padding:6px 10px">
        <span style="font-size:.72rem;opacity:.5">🔍</span>
        <input placeholder="Search…" style="font-size:.78rem">
      </div>
    </div>

    <div class="chat-list-body">
      <!-- Recent sessions -->
      {#if $chatSessions.length > 0}
        <div class="list-label">Recent</div>
        {#each $chatSessions as session (session.id)}
          <div class="chat-item" class:active={activeSessionId === session.id} role="button" tabindex="0" onclick={() => selectSession(session)} onkeydown={(e) => e.key === 'Enter' && selectSession(session)}>
            <span class="chat-item-icon">{session.targetIcon}</span>
            <div class="chat-item-info">
              <div class="chat-item-name">{session.title}</div>
              <div class="chat-item-sub">{session.targetName} · {session.updatedAt.slice(0, 10)}</div>
              {#if session.lastMessage}
                <div class="chat-item-preview">{session.lastMessage}</div>
              {/if}
            </div>
            <button class="chat-item-delete" onclick={(e) => handleDeleteSession(session.id, e)} title="Delete chat">✕</button>
          </div>
        {/each}
      {/if}

      <!-- Cloud Deployments -->
      <div class="list-label" style="margin-top:12px">Deployments</div>
      {#if $authStatus.signed_in && deployments.length > 0}
        {#each deployments as d (d.name)}
          <button class="chat-item" class:active={chatTarget?.kind === 'deployment' && chatTarget?.id === d.name} onclick={() => selectDeployment(d)}>
            <span class="chat-item-icon">🧠</span>
            <div class="chat-item-info">
              <div class="chat-item-name">{d.properties.model?.name ?? d.name}</div>
              <div class="chat-item-sub">Foundry · {d.sku?.name || 'Deployment'}</div>
            </div>
            <div class="dot dot-green" style="margin-left:auto"></div>
          </button>
        {/each}
      {:else}
        <div style="padding:8px 10px;font-size:.78rem;color:var(--text-3)">{$authStatus.signed_in ? 'No deployments found.' : 'Sign in to see deployments.'}</div>
      {/if}

      <!-- Foundry Agents -->
      {#if liveAgents.length > 0}
        <div class="list-label" style="margin-top:12px">Agents</div>
        {#each liveAgents as agent (agent.id)}
          <button class="chat-item" class:active={chatTarget?.kind === 'agent' && chatTarget?.id === agent.id} onclick={() => selectAgent(agent)}>
            <span class="chat-item-icon">🤖</span>
            <div class="chat-item-info">
              <div class="chat-item-name">{agent.name || agent.id}</div>
              <div class="chat-item-sub">Foundry Agent · {agent.model}</div>
            </div>
            <div class="dot dot-blue" style="margin-left:auto"></div>
          </button>
        {/each}
      {/if}

      <!-- Local Models -->
      {#if $localAgents.length > 0}
        <div class="list-label" style="margin-top:12px">Local Models</div>
        {#each $localAgents as agent (agent.id)}
          <button class="chat-item" class:active={chatTarget?.kind === 'local' && chatTarget?.id === agent.id} onclick={() => selectLocalAgent(agent)}>
            <span class="chat-item-icon">🏠</span>
            <div class="chat-item-info">
              <div class="chat-item-name">{agent.name}</div>
              <div class="chat-item-sub">{agent.runtime} · {agent.model ?? 'Local'}</div>
            </div>
            <div class="dot dot-green" style="margin-left:auto"></div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Chat Area -->
  <div class="chat-area">
    <div class="chat-topbar">
      <div class="res-icon" style="width:32px;height:32px;background:rgba(26,137,240,.12);font-size:.85rem">{topbarIcon}</div>
      <div>
        <div style="font-weight:600;font-size:.88rem">{topbarName}</div>
        <div style="font-size:.68rem;color:var(--text-3)">{topbarDetail}</div>
      </div>
      <span class="badge {chatTargetBadgeClass}" style="margin-left:4px">{chatTargetBadge}</span>
      <div style="margin-left:auto;display:flex;gap:4px;align-items:center">
        {#if messages.length > 0}
          <span class="token-counter" title="Estimated tokens">~{totalTokens.toLocaleString()} tokens</span>
        {/if}
        <button class="btn-icon" title="New chat" onclick={startNewChat}>🗒️</button>
      </div>
    </div>

    <div class="chat-messages" bind:this={chatMessagesEl}>
      {#if messages.length === 0}
        <div class="chat-empty-state">
          <div class="chat-empty-icon">{topbarIcon}</div>
          <div class="chat-empty-title">{chatTarget ? `Chat with ${topbarName}` : 'Start a conversation'}</div>
          <div class="chat-empty-desc">
            {#if chatTarget?.kind === 'agent'}
              This is a Foundry Agent. It uses threads and runs to process your messages.
            {:else if chatTarget?.kind === 'local'}
              Chatting locally — no data leaves your machine.
            {:else if chatTarget}
              Streaming responses from Azure AI Foundry.
            {:else}
              Select a deployment, agent, or local model from the sidebar.
            {/if}
          </div>
          {#if chatTarget}
            <div class="suggestion-chips">
              {#each suggestions as s}
                <button class="suggestion-chip" onclick={() => handleSuggestion(s)}>{s}</button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      {#each messages as msg, i (msg.id)}
        <div class="msg {msg.role === 'user' ? 'user' : 'ai'}" class:msg-animate={true}>
          <div class="msg-avatar">{msg.role === 'user' ? '👤' : topbarIcon}</div>
          <div class="msg-content">
            <div class="msg-header">
              <span class="msg-sender">{msg.role === 'user' ? 'You' : topbarName}</span>
              <span class="msg-time">{msg.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
            </div>
            <div class="msg-body">
              <MarkdownRenderer content={msg.content} isUser={msg.role === 'user'} />
              {#if isStreaming && i === messages.length - 1 && msg.role === 'assistant' && msg.content.length > 0}
                <span class="streaming-cursor">|</span>
              {/if}
            </div>
            {#if msg.role === 'assistant' && msg.content && (!isStreaming || i !== messages.length - 1)}
              <div class="msg-actions">
                <button class="msg-action-btn" onclick={() => copyMessage(msg.content)} title="Copy">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
                  Copy
                </button>
                <button class="msg-action-btn" onclick={() => regenerateMessage(i)} title="Regenerate">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 4v6h6"/><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/></svg>
                  Retry
                </button>
              </div>
            {/if}
          </div>
        </div>
      {/each}

      {#if isStreaming && messages.length > 0 && messages[messages.length - 1].content === ''}
        <div class="msg ai">
          <div class="msg-avatar">{topbarIcon}</div>
          <div class="msg-content">
            <div class="msg-header">
              <span class="msg-sender">{topbarName}</span>
            </div>
            <div class="msg-body">
              <div class="typing"><span></span><span></span><span></span></div>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <div class="chat-input-area">
      {#if isStreaming}
        <button class="stop-btn" onclick={stopGeneration}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="1"/></svg>
          Stop generating
        </button>
      {/if}
      <div class="chat-input-bar">
        <textarea
          bind:this={textareaEl}
          placeholder={chatTarget ? `Message ${topbarName}…` : 'Select a chat target first…'}
          rows="1"
          bind:value={inputText}
          onkeydown={handleKey}
          oninput={autoResizeTextarea}
          disabled={!chatTarget}
        ></textarea>
        <button class="send-btn" onclick={send} title="Send (Enter)" disabled={isStreaming || !chatTarget || !inputText.trim()}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>
        </button>
      </div>
      <div class="input-hint">
        {#if chatTarget?.kind === 'local'}
          🔒 Running locally · Enter to send, Shift+Enter for new line
        {:else if chatTarget?.kind === 'agent'}
          🤖 Agent mode · Enter to send, Shift+Enter for new line
        {:else}
          Enter to send, Shift+Enter for new line
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .chat-shell { display: flex; height: 100%; }

  /* ---- Sidebar ---- */
  .chat-list {
    width: 280px; flex-shrink: 0; border-right: 1px solid var(--border);
    display: flex; flex-direction: column; background: var(--bg-1);
  }
  .chat-list-header { padding: 16px; border-bottom: 1px solid var(--border); }
  .chat-list-body { flex: 1; overflow-y: auto; padding: 8px; }
  .list-label { font-size: .62rem; font-weight: 600; text-transform: uppercase; letter-spacing: .06em; color: var(--text-3); padding: 4px 8px 6px; }
  .chat-item {
    display: flex; align-items: center; gap: 10px; padding: 9px 10px;
    border-radius: var(--radius-sm); cursor: pointer; transition: background var(--transition);
    width: 100%; text-align: left; position: relative;
  }
  .chat-item:hover { background: var(--bg-hover); }
  .chat-item.active { background: var(--bg-selected); }
  .chat-item-icon { font-size: .92rem; flex-shrink: 0; }
  .chat-item-info { flex: 1; min-width: 0; }
  .chat-item-name { font-size: .82rem; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .chat-item-sub { font-size: .65rem; color: var(--text-3); }
  .chat-item-preview {
    font-size: .68rem; color: var(--text-3); margin-top: 2px;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 180px;
  }
  .chat-item-delete {
    opacity: 0; font-size: .68rem; color: var(--text-3); padding: 2px 4px;
    border-radius: 3px; transition: all 0.1s; background: transparent; border: none; cursor: pointer; flex-shrink: 0;
  }
  .chat-item:hover .chat-item-delete { opacity: 1; }
  .chat-item-delete:hover { color: var(--error); background: rgba(239,68,68,.1); }

  /* ---- Chat Area ---- */
  .chat-area { flex: 1; display: flex; flex-direction: column; min-width: 0; }
  .chat-topbar { display: flex; align-items: center; gap: 12px; padding: 12px 20px; border-bottom: 1px solid var(--border); flex-shrink: 0; }
  .chat-messages {
    flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 0;
    scroll-behavior: smooth;
  }

  /* ---- Messages (full-width, ChatGPT-style) ---- */
  .msg {
    display: flex; gap: 16px; padding: 20px 28px;
    max-width: 100%; animation: msg-fade-in 0.25s ease-out;
  }
  .msg.user {
    background: transparent;
  }
  .msg.ai {
    background: var(--bg-3);
  }
  @keyframes msg-fade-in {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .msg-avatar {
    width: 28px; height: 28px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    font-size: .78rem; flex-shrink: 0; margin-top: 2px;
  }
  .msg-content {
    flex: 1; min-width: 0; max-width: 760px;
  }
  .msg-header {
    display: flex; align-items: baseline; gap: 8px; margin-bottom: 4px;
  }
  .msg-sender {
    font-size: .78rem; font-weight: 600; color: var(--text-1);
  }
  .msg-time {
    font-size: .62rem; color: var(--text-3);
  }
  .msg-body {
    font-size: .88rem; line-height: 1.65; color: var(--text-1);
  }

  /* Streaming cursor */
  .streaming-cursor {
    display: inline;
    font-weight: 200;
    animation: cursor-blink 0.8s infinite;
    color: var(--brand);
    font-size: 1.05em;
  }
  @keyframes cursor-blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }

  /* ---- Message actions toolbar ---- */
  .msg-actions {
    display: flex; gap: 4px; margin-top: 8px; opacity: 0; transition: opacity 0.15s;
  }
  .msg:hover .msg-actions { opacity: 1; }
  .msg-action-btn {
    display: inline-flex; align-items: center; gap: 4px;
    padding: 3px 8px; border-radius: 4px; font-size: .68rem; font-weight: 500;
    color: var(--text-3); background: transparent; border: 1px solid transparent;
    cursor: pointer; transition: all 0.1s;
  }
  .msg-action-btn:hover {
    color: var(--text-1); background: var(--bg-hover); border-color: var(--border);
  }
  .msg-action-btn svg { flex-shrink: 0; }

  /* ---- Empty state + suggestions ---- */
  .chat-empty-state {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    flex: 1; gap: 12px; padding: 4rem 2rem; text-align: center;
  }
  .chat-empty-icon { font-size: 2.5rem; }
  .chat-empty-title { font-size: 1.2rem; font-weight: 600; color: var(--text-1); }
  .chat-empty-desc { font-size: .85rem; color: var(--text-3); max-width: 420px; line-height: 1.5; }
  .suggestion-chips {
    display: flex; flex-wrap: wrap; gap: 8px; margin-top: 16px; justify-content: center;
  }
  .suggestion-chip {
    padding: 8px 14px; border-radius: var(--radius-md); font-size: .8rem;
    border: 1px solid var(--border); color: var(--text-2); background: var(--bg-1);
    cursor: pointer; transition: all 0.15s; max-width: 260px; text-align: left;
  }
  .suggestion-chip:hover {
    border-color: var(--brand); color: var(--brand); background: var(--bg-selected);
  }

  /* ---- Input area ---- */
  .chat-input-area {
    border-top: 1px solid var(--border); background: var(--bg-1);
    display: flex; flex-direction: column; align-items: center; padding: 12px 20px 8px;
  }
  .chat-input-bar {
    display: flex; gap: 10px; align-items: flex-end; width: 100%; max-width: 760px;
  }
  .chat-input-bar textarea {
    flex: 1; resize: none; min-height: 42px; max-height: 160px;
    padding: 10px 14px; border-radius: var(--radius-lg);
    background: var(--bg-2); border: 1px solid var(--border); line-height: 1.5; font-size: .88rem;
    transition: border-color 0.15s, box-shadow 0.15s;
  }
  .chat-input-bar textarea:focus {
    border-color: var(--brand); box-shadow: 0 0 0 3px rgba(26,137,240,.12);
  }
  .chat-input-bar textarea:disabled {
    opacity: 0.5; cursor: not-allowed;
  }
  .send-btn {
    width: 42px; height: 42px; border-radius: 50%; background: var(--brand); color: #fff;
    display: flex; align-items: center; justify-content: center; flex-shrink: 0;
    transition: all 0.15s;
  }
  .send-btn:hover:not(:disabled) { filter: brightness(1.1); transform: scale(1.04); }
  .send-btn:disabled { opacity: .35; cursor: not-allowed; }
  .input-hint {
    font-size: .62rem; color: var(--text-3); margin-top: 6px; text-align: center;
  }

  /* ---- Stop button ---- */
  .stop-btn {
    display: inline-flex; align-items: center; gap: 6px; margin-bottom: 8px;
    padding: 6px 14px; border-radius: 99px; font-size: .75rem; font-weight: 500;
    border: 1px solid var(--border); color: var(--text-2); background: var(--bg-1);
    cursor: pointer; transition: all 0.15s;
  }
  .stop-btn:hover {
    border-color: var(--error); color: var(--error); background: rgba(239,68,68,.06);
  }

  /* ---- Typing indicator ---- */
  .typing { display: flex; gap: 4px; padding: 6px 0; }
  .typing span {
    width: 6px; height: 6px; border-radius: 50%; background: var(--text-3);
    animation: typing-blink 1.4s infinite ease-in-out;
  }
  .typing span:nth-child(2) { animation-delay: .2s; }
  .typing span:nth-child(3) { animation-delay: .4s; }
  @keyframes typing-blink { 0%,60%,100% { opacity: .3; transform: translateY(0); } 30% { opacity: 1; transform: translateY(-4px); } }

  /* ---- Token counter ---- */
  .token-counter {
    font-size: .68rem; color: var(--text-3); padding: 2px 8px;
    border-radius: 99px; background: var(--bg-3); white-space: nowrap;
  }
</style>
