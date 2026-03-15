<script lang="ts">
  import { agents, activityFeed } from '../stores/data';
  import { navigateTo } from '../stores/navigation';
</script>

<div class="page-header">
  <div class="page-title">Good evening, MJ 👋</div>
  <div class="page-subtitle">Your unified view across Foundry, Copilot Studio, M365, and local agents</div>
</div>

<div class="stats-row">
  <div class="stat">
    <div class="stat-val">7</div>
    <div class="stat-label">Total Agents</div>
    <div class="stat-sub" style="color:var(--text-3)">3 Foundry · 2 Studio · 1 M365 · 1 Local</div>
  </div>
  <div class="stat">
    <div class="stat-val" style="color:var(--success)">5 / 10</div>
    <div class="stat-label">Models Online</div>
    <div class="stat-sub">3 cloud · 2 local</div>
  </div>
  <div class="stat">
    <div class="stat-val">2</div>
    <div class="stat-label">Active Runs</div>
    <div class="stat-sub up">↑ support-agent, batch-classify</div>
  </div>
  <div class="stat">
    <div class="stat-val">$342</div>
    <div class="stat-label">Month Spend</div>
    <div class="stat-sub down">↑ 12% vs Feb</div>
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
  <div class="agent-grid">
    {#each agents.slice(0, 4) as agent}
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
</div>

<!-- Activity Feed -->
<div class="section">
  <div class="section-header">
    <div class="section-title">Recent Activity</div>
  </div>
  <div class="card" style="padding:8px 16px">
    <div class="feed">
      {#each activityFeed as item}
        <div class="feed-item">
          <div class="dot dot-{item.color}" style="margin-top:6px"></div>
          <div class="feed-body">
            <div class="feed-text">{@html item.text}</div>
            <div class="feed-time">{item.time} · {item.source}</div>
          </div>
        </div>
      {/each}
    </div>
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
