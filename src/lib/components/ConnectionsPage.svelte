<script lang="ts">
  import { connections } from '../stores/data';

  const azure = connections.filter(c => c.category === 'azure');
  const m365 = connections.filter(c => c.category === 'm365');
  const local = connections.filter(c => c.category === 'local');

  const catIcon: Record<string, string[]> = {
    azure: ['🧠', '🔎', '💾', '🔑'],
    m365: ['👤', '✨'],
    local: ['🏠', '🏠'],
  };
  const catColor: Record<string, string[]> = {
    azure: ['rgba(26,137,240,.1)', 'rgba(245,158,11,.1)', 'rgba(99,102,241,.1)', 'rgba(239,68,68,.1)'],
    m365: ['rgba(99,102,241,.1)', 'rgba(236,72,153,.1)'],
    local: ['rgba(245,158,11,.1)', 'rgba(245,158,11,.1)'],
  };
</script>

<div class="page-header">
  <div class="page-title">Connections</div>
  <div class="page-subtitle">Infrastructure linking your agents to data, compute, and services</div>
</div>

{#each [{ title: 'Azure Resources', items: azure, cat: 'azure' }, { title: 'Microsoft 365', items: m365, cat: 'm365' }, { title: 'Local Services', items: local, cat: 'local' }] as group}
  {#if group.items.length}
    <div class="section">
      <div class="section-header"><div class="section-title">{group.title}</div></div>
      <div class="card" style="padding:6px 0">
        <div class="res-list">
          {#each group.items as conn, i (conn.id)}
            <div class="res-row">
              <div class="res-icon" style="background:{catColor[group.cat]?.[i] || 'var(--bg-3)'}">{catIcon[group.cat]?.[i] || '🔗'}</div>
              <div class="res-info">
                <div class="res-name">{conn.name}</div>
                <div class="res-meta">{conn.description}</div>
              </div>
              <span class="badge {conn.status === 'connected' ? 'badge-green' : 'badge-muted'}">
                {conn.status === 'connected' ? '● Connected' : 'Configured'}
              </span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
{/each}
