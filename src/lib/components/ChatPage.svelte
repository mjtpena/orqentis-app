<script lang="ts">
  import { chatSessions, models as mockModels, agents as mockAgents } from '../stores/data';
  import { activeEndpoint, authStatus } from '../stores/auth';
  import * as api from '../services/api';
  import type { FoundryDeployment, ChatMessagePayload } from '../services/api';
  import type { ChatMessage } from '../types';
  import { listen } from '@tauri-apps/api/event';

  let messages = $state<ChatMessage[]>([
    { id: '1', role: 'user', content: 'Summarize the Q4 2025 financial report. Focus on revenue growth, key risks, and strategic outlook.', timestamp: new Date() },
    { id: '2', role: 'assistant', content: `**Q4 2025 Financial Summary**

**Revenue Growth:** Total revenue increased 18% YoY to $4.2B, driven by cloud services (+24%) and AI platform adoption (+31%).

**Key Risks:**
- Increased competition in AI infrastructure
- Regulatory uncertainty in EU and APAC
- GPU supply chain constraints

**Strategic Outlook:** Management expects 15-20% revenue growth in FY2026 with focus on agentic AI and expanded Copilot integrations.`, timestamp: new Date() },
    { id: '3', role: 'user', content: 'What about operating margins vs Q3?', timestamp: new Date() },
    { id: '4', role: 'assistant', content: `Operating margin improved to **42.3%** in Q4, up from 40.1% in Q3 (+220 bps):

- **Gross margin:** 69.8% → 71.2% (cloud efficiency)
- **R&D:** Flat at 14.2% despite AI investments
- **S&M:** Down 80 bps to 13.7% via automation

Highest operating margin in 8 quarters.`, timestamp: new Date() },
  ]);

  let inputText = $state('');
  let isStreaming = $state(false);
  let chatMessagesEl: HTMLDivElement;

  // Live deployments for model selector
  let deployments = $state<FoundryDeployment[]>([]);
  let selectedDeployment = $state('');
  let deploymentsLoading = $state(false);

  $effect(() => {
    const endpoint = $activeEndpoint;
    const status = $authStatus;
    if (status.signed_in && endpoint) {
      deploymentsLoading = true;
      api.listFoundryDeployments(endpoint)
        .then(data => {
          deployments = data;
          if (data.length > 0 && !selectedDeployment) {
            selectedDeployment = data[0].id;
          }
        })
        .catch(() => {})
        .finally(() => { deploymentsLoading = false; });
    } else {
      deployments = [];
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
      // Mock response when not connected
      setTimeout(() => {
        messages = [...messages, { id: crypto.randomUUID(), role: 'assistant', content: 'This is a prototype response. Sign in and select a deployment to chat with a real model.', timestamp: new Date() }];
        isStreaming = false;
        scrollToBottom();
      }, 1500);
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); send(); }
  }

  const chatModels = mockModels.filter(m => m.status === 'online' && m.capabilities?.includes('chat'));

  let topbarName = $derived(
    $authStatus.signed_in && selectedDeployment
      ? deployments.find(d => d.id === selectedDeployment)?.model.name ?? selectedDeployment
      : 'gpt-4o'
  );

  let topbarDetail = $derived(
    $authStatus.signed_in && selectedDeployment
      ? `Foundry · ${deployments.find(d => d.id === selectedDeployment)?.model.version ?? 'Live'}`
      : 'Foundry · v2024-08-06 · East US'
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
      {#each chatSessions as session (session.id)}
        <button class="chat-item" class:active={session.id === 's1'}>
          <span class="chat-item-icon">{session.targetIcon}</span>
          <div class="chat-item-info">
            <div class="chat-item-name">{session.title}</div>
            <div class="chat-item-sub">{session.targetName} · {session.updatedAt}</div>
          </div>
        </button>
      {/each}

      <div class="list-label" style="margin-top:12px">Targets</div>
      {#if $authStatus.signed_in && deployments.length > 0}
        {#each deployments as d (d.id)}
          <button class="chat-item" class:active={selectedDeployment === d.id} onclick={() => selectedDeployment = d.id}>
            <span class="chat-item-icon">🧠</span>
            <div class="chat-item-info">
              <div class="chat-item-name">{d.model.name}</div>
              <div class="chat-item-sub">Foundry · {d.sku?.name || 'Deployment'}</div>
            </div>
            <div class="dot dot-green" style="margin-left:auto"></div>
          </button>
        {/each}
      {:else}
        {#each chatModels as model (model.id)}
          <button class="chat-item">
            <span class="chat-item-icon">{model.source === 'local' ? '🏠' : '🧠'}</span>
            <div class="chat-item-info">
              <div class="chat-item-name">{model.name}</div>
              <div class="chat-item-sub">{model.source === 'local' ? 'Local · Ollama' : `Foundry · ${model.sku || 'Online'}`}</div>
            </div>
            <div class="dot dot-green" style="margin-left:auto"></div>
          </button>
        {/each}
      {/if}
      {#each mockAgents.filter(a => ['active', 'running', 'published'].includes(a.status)) as agent (agent.id)}
        <button class="chat-item">
          <span class="chat-item-icon">{agent.source === 'foundry' ? '🤖' : agent.source === 'studio' ? '✨' : agent.source === 'local' ? '🏠' : '👤'}</span>
          <div class="chat-item-info">
            <div class="chat-item-name">{agent.name}</div>
            <div class="chat-item-sub">{agent.source === 'foundry' ? `Foundry Agent · ${agent.model}` : agent.source === 'studio' ? 'Copilot Studio' : agent.source === 'local' ? `Local · ${agent.runtime}` : 'M365 Copilot'}</div>
          </div>
        </button>
      {/each}
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
          {#each deployments as d (d.id)}
            <option value={d.id}>{d.model.name}{d.model.version ? ` (${d.model.version})` : ''}</option>
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
