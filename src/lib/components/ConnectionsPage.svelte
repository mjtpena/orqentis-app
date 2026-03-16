<script lang="ts">
  import { activeEndpoint, activeHub, authStatus } from '../stores/auth';
  import * as api from '../services/api';
  import type { FoundryConnection, ArmConnection, HubDetail } from '../services/api';
  import type { Connection } from '../types';

  let liveConnections = $state<Connection[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  function detectCategory(cat: string): Connection['category'] {
    const lower = cat.toLowerCase();
    if (lower.includes('azure') || lower.includes('cognitive') || lower.includes('openai') || lower.includes('search') || lower.includes('storage') || lower.includes('key_vault')) return 'azure';
    if (lower.includes('graph') || lower.includes('m365') || lower.includes('dynamics') || lower.includes('sharepoint')) return 'm365';
    return 'azure';
  }

  function mapArmConnection(c: ArmConnection): Connection {
    return {
      id: c.name,
      name: c.name,
      description: `${c.properties.category ?? 'Unknown'}${c.properties.target ? ` · ${c.properties.target}` : ''}`,
      category: detectCategory(c.properties.category ?? ''),
      status: 'connected',
    };
  }

  function mapConnection(c: FoundryConnection): Connection {
    return {
      id: c.id ?? c.name,
      name: c.name,
      description: `${c.properties.category ?? 'Unknown'}${c.properties.target ? ` · ${c.properties.target}` : ''}`,
      category: detectCategory(c.properties.category ?? ''),
      status: 'connected',
    };
  }

  $effect(() => {
    const endpoint = $activeEndpoint;
    const hub = $activeHub;
    const status = $authStatus;
    if (!status.signed_in || !endpoint) {
      liveConnections = [];
      return;
    }

    // Use ARM connections from hub discovery (always available, no extra API call)
    if (hub && hub.connections && hub.connections.length > 0) {
      liveConnections = hub.connections.map(mapArmConnection);
      return;
    }

    // Fallback to data-plane API
    loading = true;
    error = null;
    api.listConnections(endpoint)
      .then(data => { liveConnections = data.map(mapConnection); })
      .catch(e => {
        console.warn('[connections] data-plane API failed:', e);
        liveConnections = [];
      })
      .finally(() => { loading = false; });
  });

  function retry() {
    // re-trigger the effect by reading the stores
    const endpoint = $activeEndpoint;
    if (!endpoint) return;
    loading = true;
    error = null;
    api.listConnections(endpoint)
      .then(data => { liveConnections = data.map(mapConnection); })
      .catch(e => { error = e.toString(); })
      .finally(() => { loading = false; });
  }

  let allConnections = $derived(liveConnections);

  let azure = $derived(allConnections.filter(c => c.category === 'azure'));
  let m365 = $derived(allConnections.filter(c => c.category === 'm365'));
  let local = $derived(allConnections.filter(c => c.category === 'local'));

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

{#if !$authStatus.signed_in}
  <div class="not-connected-banner">
    <span>🔌</span>
    <span style="font-size:.85rem;color:var(--text-2)">Sign in to see your Foundry connections. Showing sample data.</span>
  </div>
{/if}

{#if loading}
  <div class="loading-state">
    <div class="spinner"></div>
    <p>Loading connections…</p>
  </div>
{:else if error}
  <div class="error-state">
    <p>⚠️ {error}</p>
    <button class="btn btn-outline" onclick={retry}>Retry</button>
  </div>
{:else}
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

  {#if allConnections.length === 0}
    <div class="empty-state">
      <p>No connections found.</p>
    </div>
  {/if}
{/if}
