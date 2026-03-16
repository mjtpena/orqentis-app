<script lang="ts">
  import { navigateTo } from '../stores/navigation';
  import { activeEndpoint, authStatus, hubs, armDeployments, activeHub, studioAgents, m365Agents, localAgents } from '../stores/auth';
  import { activityFeed } from '../stores/data';
  import * as api from '../services/api';
  import type { Agent } from '../types';
  import type { FoundryAgent, StudioBot, M365Agent, LocalAgent } from '../services/api';

  let foundryAgents = $state<Agent[]>([]);
  let loading = $state(false);

  function mapFoundryAgent(fa: FoundryAgent): Agent {
    return {
      id: fa.id,
      name: fa.name ?? fa.id,
      description: fa.instructions || `Model: ${fa.model}`,
      source: 'foundry',
      model: fa.model,
      status: 'active',
      tools: fa.tools.map(t => t.type),
    };
  }

  function mapStudioBot(b: StudioBot): Agent {
    return { id: b.id, name: b.name, description: b.description ?? 'Copilot Studio bot', source: 'studio', status: b.status === 'published' ? 'published' : 'draft' };
  }

  function mapM365Agent(a: M365Agent): Agent {
    return { id: a.id, name: a.name, description: a.description ?? 'M365 Copilot agent', source: 'm365', status: 'active' };
  }

  function mapLocalAgent(a: LocalAgent): Agent {
    return { id: a.id, name: a.name, description: a.description ?? a.runtime, source: 'local', model: a.model ?? undefined, runtime: a.runtime, status: 'running' };
  }

  $effect(() => {
    const endpoint = $activeEndpoint;
    const status = $authStatus;
    if (status.signed_in && endpoint) {
      loading = true;
      api.listAgents(endpoint)
        .then((agents) => { foundryAgents = agents.map(mapFoundryAgent); })
        .catch((e) => { console.error('[HomePage] Failed to load agents:', e); })
        .finally(() => { loading = false; });
    } else {
      foundryAgents = [];
    }
  });

  let allAgents = $derived([
    ...foundryAgents,
    ...$studioAgents.map(mapStudioBot),
    ...$m365Agents.map(mapM365Agent),
    ...$localAgents.map(mapLocalAgent),
  ]);

  let modelCount = $derived($armDeployments.length);
  let projectCount = $derived($activeHub?.projects?.length ?? 0);

  let greeting = $derived(
    $authStatus.signed_in && $authStatus.user_name
      ? `Welcome, ${$authStatus.user_name} 👋`
      : 'Welcome to Orqentis 👋'
  );
</script>

<div class="page-header">
  <div class="page-title">{greeting}</div>
  <div class="page-subtitle">Your unified view across Foundry, Copilot Studio, M365, and local agents</div>
</div>

{#if !$authStatus.signed_in}
  <div class="not-connected-banner">
    <span>🔌</span>
    <span style="font-size:.85rem;color:var(--text-2)">Sign in to see your live resources. Showing sample data.</span>
  </div>
{/if}

<div class="stats-row">
  <div class="stat">
    <div class="stat-val">{$authStatus.signed_in ? allAgents.length : '—'}</div>
    <div class="stat-label">Total Agents</div>
    <div class="stat-sub" style="color:var(--text-3)">{$authStatus.signed_in ? `${foundryAgents.length} Foundry · ${$studioAgents.length} Studio · ${$m365Agents.length} M365 · ${$localAgents.length} Local` : 'Sign in to view'}</div>
  </div>
  <div class="stat">
    <div class="stat-val" style="color:var(--success)">{$authStatus.signed_in ? modelCount : '—'}</div>
    <div class="stat-label">Deployments</div>
    <div class="stat-sub">{$authStatus.signed_in ? 'Cloud model deployments' : 'Sign in to view'}</div>
  </div>
  <div class="stat">
    <div class="stat-val">{$authStatus.signed_in ? $hubs.length : '—'}</div>
    <div class="stat-label">Hubs</div>
    <div class="stat-sub">{$authStatus.signed_in ? `${projectCount} project${projectCount !== 1 ? 's' : ''}` : 'Sign in to view'}</div>
  </div>
  <div class="stat">
    <div class="stat-val">{$authStatus.signed_in ? $localAgents.length : '—'}</div>
    <div class="stat-label">Local Runtimes</div>
    <div class="stat-sub">{$authStatus.signed_in ? ($localAgents.length > 0 ? 'Ollama / LM Studio / etc.' : 'None detected') : 'Sign in to view'}</div>
  </div>
</div>

<!-- Quick Actions -->
<div class="section">
  <div class="section-header">
    <div class="section-title">Quick Actions</div>
  </div>
  <div style="display:flex;gap:10px;flex-wrap:wrap">
    <button class="btn btn-outline" onclick={() => navigateTo('chat')}>💬 New Chat</button>
    <button class="btn btn-outline" onclick={() => navigateTo('agents')}>🤖 Create Agent</button>
    <button class="btn btn-outline" onclick={() => navigateTo('knowledge')}>📤 Upload Knowledge</button>
    <button class="btn btn-outline" onclick={() => navigateTo('runs')}>▶️ View Runs</button>
  </div>
</div>

<!-- Agents -->
<div class="section">
  <div class="section-header">
    <div class="section-title">Your Agents</div>
    <button class="section-link" onclick={() => navigateTo('agents')}>View all →</button>
  </div>
  {#if loading}
    <div class="loading-state" style="padding:1.5rem">
      <div class="spinner"></div>
      <p>Loading…</p>
    </div>
  {:else if allAgents.length === 0}
    <div class="empty-state" style="padding:1.5rem">
      <p style="color:var(--text-3);font-size:.88rem">{$authStatus.signed_in ? 'No agents found across any source.' : 'Sign in to see your agents.'}</p>
    </div>
  {:else}
    <div class="agent-grid">
      {#each allAgents.slice(0, 6) as agent}
        <button class="agent-card src-{agent.source}" onclick={() => navigateTo('chat')}>
          <div class="agent-header">
            <div class="agent-icon {agent.source}">
              {agent.source === 'foundry' ? '🤖' : agent.source === 'studio' ? '✨' : agent.source === 'm365' ? '👤' : '🏠'}
            </div>
            <div>
              <div class="agent-name">{agent.name}</div>
              <div class="agent-desc">{agent.description}</div>
            </div>
          </div>
          <div class="agent-meta">
            <span class="badge badge-{agent.source === 'foundry' ? 'blue' : agent.source === 'studio' ? 'pink' : agent.source === 'm365' ? 'purple' : 'yellow'}">
              {agent.source === 'foundry' ? 'Foundry' : agent.source === 'studio' ? 'Studio' : agent.source === 'm365' ? 'M365' : 'Local'}
            </span>
            {#if agent.model}
              <span class="badge badge-muted">{agent.model}</span>
            {/if}
            {#if agent.threads}
              <span class="badge badge-green">● {agent.threads} threads</span>
            {/if}
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<!-- Activity Feed -->
<div class="section">
  <div class="section-header">
    <div class="section-title">Recent Activity</div>
  </div>
  <div class="card" style="padding:8px 16px">
    {#if $activityFeed.length > 0}
      <div class="feed">
        {#each $activityFeed as item}
          <div class="feed-item">
            <div class="dot dot-{item.color}" style="margin-top:6px"></div>
            <div class="feed-body">
              <div class="feed-text">{@html item.text}</div>
              <div class="feed-time">{item.time} · {item.source}</div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div style="padding:16px 0;text-align:center;color:var(--text-3);font-size:.85rem">
        {$authStatus.signed_in ? 'No recent activity yet.' : 'Sign in to see activity.'}
      </div>
    {/if}
  </div>
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
  .agent-desc {
    font-size: .78rem; color: var(--text-2); line-height: 1.45;
    display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
    text-align: left;
  }
  .agent-meta { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; margin-top: 10px; }
  .feed { display: flex; flex-direction: column; gap: 2px; }
  .feed-item { display: flex; align-items: flex-start; gap: 12px; padding: 10px 0; }
  .feed-body { flex: 1; }
  .feed-text { font-size: .85rem; }
  .feed-time { font-size: .68rem; color: var(--text-3); margin-top: 2px; }
</style>
