<script lang="ts">
  import { chatSessions } from '../stores/data';
  import { activeEndpoint, authStatus, armDeployments } from '../stores/auth';
  import * as api from '../services/api';
  import type { ArmDeployment, ChatMessagePayload, FoundryAgent } from '../services/api';
  import type { ChatMessage } from '../types';
  import { listen } from '@tauri-apps/api/event';

  let messages = $state<ChatMessage[]>([]);

  let inputText = $state('');
  let isStreaming = $state(false);
  let chatMessagesEl: HTMLDivElement;

  // Live deployments from ARM discovery and agents from Foundry
  let liveAgents = $state<FoundryAgent[]>([]);
  let selectedDeployment = $state('');
  let deploymentsLoading = $state(false);

  // Use ARM deployments directly from the store
  let deployments = $derived($armDeployments);

  $effect(() => {
    const endpoint = $activeEndpoint;
    const status = $authStatus;
    if (status.signed_in && endpoint) {
      deploymentsLoading = true;
      api.listAgents(endpoint)
        .then((agents) => {
          liveAgents = agents;
        })
        .catch((e) => { console.error('[ChatPage] Failed to load agents:', e); })
        .finally(() => { deploymentsLoading = false; });
    } else {
      liveAgents = [];
    }
  });

  // Auto-select first deployment when available
  $effect(() => {
    if (deployments.length > 0 && !selectedDeployment) {
      selectedDeployment = deployments[0].name;
    }
  });

  function scrollToBottom() {
    if (chatMessagesEl) {
      requestAnimationFrame(() => { chatMessagesEl.scrollTop = chatMessagesEl.scrollHeight; });
    }
  }

  async function send() {
    const text = inputText.trim();
    if (!text || isStreaming) return;

    messages = [...messages, { id: crypto.randomUUID(), role: 'user', content: text, timestamp: new Date() }];
    inputText = '';
    isStreaming = true;
    scrollToBottom();

    const endpoint = $activeEndpoint;
    if ($authStatus.signed_in && endpoint && selectedDeployment) {
      // Live streaming chat
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
          isStreaming = false;
          scrollToBottom();
        }
      });

      const payload: ChatMessagePayload[] = messages
        .filter(m => m.role !== 'system')
        .slice(0, -1) // exclude the empty assistant placeholder
        .map(m => ({ role: m.role, content: m.content }));

      try {
        await api.sendChatMessage(endpoint, selectedDeployment, payload);
      } catch (e: any) {
        const idx = messages.findIndex(m => m.id === assistantId);
        if (idx !== -1) {
          messages[idx] = { ...messages[idx], content: `⚠️ Error: ${e}` };
          messages = [...messages];
        }
        isStreaming = false;
      } finally {
        unlisten();
        if (isStreaming) isStreaming = false;
      }
    } else {
      // Not connected — show a message
      messages = [...messages, { id: crypto.randomUUID(), role: 'assistant', content: 'Please sign in and select a deployment to start chatting with a real model.', timestamp: new Date() }];
      isStreaming = false;
      scrollToBottom();
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); send(); }
  }

  const chatModels: never[] = [];

  let topbarName = $derived(
    $authStatus.signed_in && selectedDeployment
      ? deployments.find(d => d.name === selectedDeployment)?.properties.model?.name ?? selectedDeployment
      : 'Select a model'
  );

  let topbarDetail = $derived(
    $authStatus.signed_in && selectedDeployment
      ? `Foundry · ${deployments.find(d => d.name === selectedDeployment)?.properties.model?.version ?? 'Live'}`
      : 'Sign in to connect'
  );
</script>

<div class="chat-shell">
  <!-- Sidebar -->
  <div class="chat-list">
    <div class="chat-list-header">
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:10px">
        <span style="font-weight:600;font-size:.95rem">Chat</span>
        <button class="btn-icon" style="font-size:.82rem">+</button>
      </div>
      <div class="search-box" style="padding:6px 10px">
        <span style="font-size:.72rem;opacity:.5">🔍</span>
        <input placeholder="Search…" style="font-size:.78rem">
      </div>
    </div>

    <div style="padding:8px;display:flex;gap:4px;flex-wrap:wrap;border-bottom:1px solid var(--border)">
      <button class="filter-chip active" style="font-size:.65rem;padding:3px 8px">All</button>
      <button class="filter-chip" style="font-size:.65rem;padding:3px 8px">🧠 Models</button>
      <button class="filter-chip" style="font-size:.65rem;padding:3px 8px">🤖 Agents</button>
      <button class="filter-chip" style="font-size:.65rem;padding:3px 8px">✨ Studio</button>
      <button class="filter-chip" style="font-size:.65rem;padding:3px 8px">🏠 Local</button>
    </div>

    <div class="chat-list-body">
      <div class="list-label">Recent</div>
      {#each $chatSessions as session (session.id)}
        <button class="chat-item" class:active={false}>
          <span class="chat-item-icon">{session.targetIcon}</span>
          <div class="chat-item-info">
            <div class="chat-item-name">{session.title}</div>
            <div class="chat-item-sub">{session.targetName} · {session.updatedAt}</div>
          </div>
        </button>
      {/each}
      {#if $chatSessions.length === 0}
        <div style="padding:8px 10px;font-size:.78rem;color:var(--text-3)">No chats yet. Start one below.</div>
      {/if}

      <div class="list-label" style="margin-top:12px">Deployments</div>
      {#if $authStatus.signed_in && deployments.length > 0}
        {#each deployments as d (d.name)}
          <button class="chat-item" class:active={selectedDeployment === d.name} onclick={() => selectedDeployment = d.name}>
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

      {#if liveAgents.length > 0}
        <div class="list-label" style="margin-top:12px">Agents</div>
        {#each liveAgents as agent (agent.id)}
          <button class="chat-item">
            <span class="chat-item-icon">🤖</span>
            <div class="chat-item-info">
              <div class="chat-item-name">{agent.name || agent.id}</div>
              <div class="chat-item-sub">Foundry Agent · {agent.model}</div>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Chat Area -->
  <div class="chat-area">
    <div class="chat-topbar">
      <div class="res-icon" style="width:32px;height:32px;background:rgba(26,137,240,.12);font-size:.85rem">🧠</div>
      <div>
        <div style="font-weight:600;font-size:.88rem">{topbarName}</div>
        <div style="font-size:.68rem;color:var(--text-3)">{topbarDetail}</div>
      </div>
      {#if $authStatus.signed_in && deployments.length > 1}
        <select
          bind:value={selectedDeployment}
          style="font-size:.75rem;padding:4px 8px;border-radius:var(--radius-sm);background:var(--bg-2);border:1px solid var(--border);color:var(--text-1);margin-left:4px"
        >
          {#each deployments as d (d.name)}
            <option value={d.name}>{d.properties.model?.name ?? d.name}{d.properties.model?.version ? ` (${d.properties.model.version})` : ''}</option>
          {/each}
        </select>
      {/if}
      <span class="badge badge-blue" style="margin-left:4px">{$authStatus.signed_in ? 'Live' : 'Prototype'}</span>
      <div style="margin-left:auto;display:flex;gap:4px">
        <button class="btn-icon" title="Attach Knowledge">📚</button>
        <button class="btn-icon" title="Export">📥</button>
      </div>
    </div>

    <div class="chat-messages" bind:this={chatMessagesEl}>
      {#each messages as msg (msg.id)}
        <div class="msg {msg.role === 'user' ? 'user' : 'ai'}">
          <div class="msg-avatar">{msg.role === 'user' ? 'MJ' : 'AI'}</div>
          <div class="msg-bubble">{@html msg.content.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>').replace(/\n- /g, '\n• ').replace(/\n/g, '<br>')}</div>
        </div>
      {/each}
      {#if isStreaming}
        <div class="msg ai">
          <div class="msg-avatar">AI</div>
          <div class="msg-bubble">
            <div class="typing"><span></span><span></span><span></span></div>
          </div>
        </div>
      {/if}
    </div>

    <div class="chat-input-bar">
      <button class="btn-icon" title="Attach file">📎</button>
      <textarea
        placeholder="Message {topbarName}… (⌘+Enter to send)"
        rows="1"
        bind:value={inputText}
        onkeydown={handleKey}
      ></textarea>
      <button class="send-btn" onclick={send} title="Send" disabled={isStreaming}>▲</button>
    </div>
  </div>
</div>

<style>
  .chat-shell { display: flex; height: 100%; }
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
    width: 100%; text-align: left;
  }
  .chat-item:hover { background: var(--bg-hover); }
  .chat-item.active { background: var(--bg-selected); }
  .chat-item-icon { font-size: .92rem; flex-shrink: 0; }
  .chat-item-info { flex: 1; min-width: 0; }
  .chat-item-name { font-size: .82rem; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .chat-item-sub { font-size: .65rem; color: var(--text-3); }

  .chat-area { flex: 1; display: flex; flex-direction: column; }
  .chat-topbar { display: flex; align-items: center; gap: 12px; padding: 14px 20px; border-bottom: 1px solid var(--border); }
  .chat-messages { flex: 1; overflow-y: auto; padding: 24px 28px; display: flex; flex-direction: column; gap: 18px; }

  .msg { display: flex; gap: 12px; max-width: 680px; }
  .msg.user { margin-left: auto; flex-direction: row-reverse; }
  .msg-avatar {
    width: 30px; height: 30px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    font-size: .68rem; font-weight: 700; flex-shrink: 0;
  }
  .msg.user .msg-avatar { background: var(--brand); color: #fff; }
  .msg.ai .msg-avatar { background: var(--gradient); color: #fff; }
  .msg-bubble { padding: 11px 16px; border-radius: var(--radius-md); font-size: .88rem; line-height: 1.6; }
  .msg.user .msg-bubble { background: var(--brand); color: #fff; border-bottom-right-radius: 3px; }
  .msg.ai .msg-bubble { background: var(--bg-2); border: 1px solid var(--border); border-bottom-left-radius: 3px; }

  .chat-input-bar {
    display: flex; gap: 10px; align-items: flex-end; padding: 16px 20px;
    border-top: 1px solid var(--border); background: var(--bg-1);
  }
  .chat-input-bar textarea {
    flex: 1; resize: none; min-height: 38px; max-height: 100px;
    padding: 8px 14px; border-radius: var(--radius-md);
    background: var(--bg-2); border: 1px solid var(--border); line-height: 1.5; font-size: .88rem;
  }
  .chat-input-bar textarea:focus { border-color: var(--brand); box-shadow: 0 0 0 3px rgba(26,137,240,.14); }
  .send-btn {
    width: 38px; height: 38px; border-radius: 50%; background: var(--brand); color: #fff;
    display: flex; align-items: center; justify-content: center; flex-shrink: 0; font-size: .88rem;
  }
  .send-btn:hover { filter: brightness(1.1); }
  .send-btn:disabled { opacity: .5; cursor: not-allowed; }

  .typing { display: flex; gap: 4px; padding: 6px 0; }
  .typing span {
    width: 6px; height: 6px; border-radius: 50%; background: var(--text-3);
    animation: blink 1.4s infinite ease-in-out;
  }
  .typing span:nth-child(2) { animation-delay: .2s; }
  .typing span:nth-child(3) { animation-delay: .4s; }
  @keyframes blink { 0%,60%,100% { opacity: .3; transform: translateY(0); } 30% { opacity: 1; transform: translateY(-4px); } }
</style>
