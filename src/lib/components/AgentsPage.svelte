<script lang="ts">
  import { agents } from '../stores/data';
  import { navigateTo } from '../stores/navigation';

  let filter = $state('all');

  let filtered = $derived(
    filter === 'all' ? agents : agents.filter(a => a.source === filter)
  );

  const sourceLabel: Record<string, string> = { foundry: 'Foundry', studio: 'Studio', m365: 'M365', local: 'Local' };
  const sourceIcon: Record<string, string> = { foundry: '🤖', studio: '✨', m365: '👤', local: '🏠' };
  const sourceBadge: Record<string, string> = { foundry: 'badge-blue', studio: 'badge-pink', m365: 'badge-purple', local: 'badge-yellow' };
  const statusBadge: Record<string, string> = { active: 'badge-green', idle: 'badge-muted', published: 'badge-green', draft: 'badge-yellow', running: 'badge-green', offline: 'badge-red' };
</script>

<div class="page-header">
  <div style="display:flex;align-items:center;justify-content:space-between">
    <div>
      <div class="page-title">Agents</div>
      <div class="page-subtitle">All agents across Foundry, Copilot Studio, M365, and local runtimes</div>
    </div>
    <button class="btn btn-primary">+ Create Agent</button>
  </div>
</div>

<div style="display:flex;gap:6px;margin-bottom:18px;flex-wrap:wrap">
  <button class="filter-chip" class:active={filter === 'all'} onclick={() => filter = 'all'}>All ({agents.length})</button>
  {#each ['foundry', 'studio', 'm365', 'local'] as src}
    <button class="filter-chip" class:active={filter === src} onclick={() => filter = src}>
      {sourceIcon[src]} {sourceLabel[src]} ({agents.filter(a => a.source === src).length})
    </button>
  {/each}
</div>

<div class="agent-grid">
  {#each filtered as agent (agent.id)}
    <div class="agent-card src-{agent.source}">
      <div class="agent-header">
        <div class="agent-icon {agent.source}">{sourceIcon[agent.source]}</div>
        <div style="flex:1;min-width:0">
          <div class="agent-name">{agent.name}</div>
          <div class="agent-desc">{agent.description}</div>
        </div>
      </div>
      <div class="agent-meta">
        <span class="badge {sourceBadge[agent.source]}">{sourceLabel[agent.source]}</span>
        {#if agent.model}<span class="badge badge-muted">{agent.model}{#if agent.runtime} · {agent.runtime}{/if}</span>{/if}
        <span class="badge {statusBadge[agent.status] || 'badge-muted'}">
          {#if ['active', 'published', 'running'].includes(agent.status)}●{/if}
          {agent.status.charAt(0).toUpperCase() + agent.status.slice(1)}
        </span>
      </div>
      <div class="agent-actions">
        <button class="btn btn-ghost" onclick={() => navigateTo('chat')}>💬 Chat</button>
        {#if agent.threads}<button class="btn btn-ghost">📋 Threads</button>{/if}
        <button class="btn btn-ghost">⚙️ {agent.source === 'local' ? 'Config' : 'Edit'}</button>
      </div>
    </div>
  {/each}
</div>

<style>
  .agent-header { display: flex; align-items: flex-start; gap: 12px; margin-bottom: 12px; }
  .agent-icon {
    width: 40px; height: 40px; border-radius: var(--radius-sm);
    display: flex; align-items: center; justify-content: center;
    font-size: 1.15rem; flex-shrink: 0;
  }
  .agent-icon.foundry { background: rgba(26,137,240,.12); }
  .agent-icon.studio { background: rgba(236,72,153,.12); }
  .agent-icon.m365 { background: rgba(99,102,241,.12); }
  .agent-icon.local { background: rgba(245,158,11,.12); }
  .agent-name { font-weight: 600; font-size: .92rem; margin-bottom: 2px; }
  .agent-desc { font-size: .78rem; color: var(--text-2); line-height: 1.45; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
  .agent-meta { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; margin-top: 10px; }
  .agent-actions { display: flex; gap: 6px; margin-top: 12px; padding-top: 12px; border-top: 1px solid var(--border); }
</style>
