<script lang="ts">
  import { models as mockModels } from '../stores/data';
  import { navigateTo } from '../stores/navigation';
  import { activeEndpoint, authStatus } from '../stores/auth';
  import * as api from '../services/api';
  import type { FoundryDeployment } from '../services/api';
  import type { Model } from '../types';

  let liveCloudModels = $state<Model[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  const mockLocalModels = mockModels.filter(m => m.source === 'local');
  const mockCloudModels = mockModels.filter(m => m.source === 'cloud');

  function mapDeployment(d: FoundryDeployment): Model {
    return {
      id: d.id,
      name: d.model.name,
      version: d.model.version,
      sku: d.sku?.name,
      source: 'cloud',
      status: d.status === 'succeeded' || d.status === 'Succeeded' ? 'online'
            : d.status === 'Failed' || d.status === 'failed' ? 'failed'
            : 'provisioning',
      capabilities: ['chat'],
    };
  }

  function loadModels(endpoint: string) {
    loading = true;
    error = null;
    api.listFoundryDeployments(endpoint)
      .then(data => { liveCloudModels = data.map(mapDeployment); })
      .catch(e => { error = e.toString(); })
      .finally(() => { loading = false; });
  }

  $effect(() => {
    const endpoint = $activeEndpoint;
    const status = $authStatus;
    if (status.signed_in && endpoint) {
      loadModels(endpoint);
    } else {
      liveCloudModels = [];
    }
  });

  function retry() {
    const endpoint = $activeEndpoint;
    if (endpoint) loadModels(endpoint);
  }

  let cloudModels = $derived($authStatus.signed_in ? liveCloudModels : mockCloudModels);
  let localModels = $derived(mockLocalModels);

  const statusBadge: Record<string, string> = { online: 'badge-green', provisioning: 'badge-yellow', failed: 'badge-red', offline: 'badge-muted' };
  const statusIcon: Record<string, string> = { online: '●', provisioning: '⟳', failed: '✗', offline: '○' };
</script>

<div class="page-header">
  <div style="display:flex;align-items:center;justify-content:space-between">
    <div>
      <div class="page-title">Models</div>
      <div class="page-subtitle">Cloud deployments and local model runtimes</div>
    </div>
    <button class="btn btn-primary">+ Deploy Model</button>
  </div>
</div>

{#if !$authStatus.signed_in}
  <div class="not-connected-banner">
    <span>🔌</span>
    <span style="font-size:.85rem;color:var(--text-2)">Sign in to see your Foundry deployments. Showing sample data.</span>
  </div>
{/if}

<div class="section">
  <div class="section-header"><div class="section-title">☁️ Azure AI Foundry</div></div>
  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading deployments…</p>
    </div>
  {:else if error}
    <div class="error-state">
      <p>⚠️ {error}</p>
      <button class="btn btn-outline" onclick={retry}>Retry</button>
    </div>
  {:else if cloudModels.length === 0}
    <div class="empty-state">
      <p>No cloud deployments found.</p>
    </div>
  {:else}
    <div class="card" style="padding:6px 0">
      <div class="res-list">
        {#each cloudModels as model (model.id)}
          <div class="res-row">
            <div class="res-icon" style="background:rgba(26,137,240,.1)">🧠</div>
            <div class="res-info">
              <div class="res-name">{model.name}</div>
              <div class="res-meta">{model.version || ''} {model.sku ? `· ${model.sku}` : ''} {model.region ? `· ${model.region}` : ''}</div>
            </div>
            <span class="badge {statusBadge[model.status]}">{statusIcon[model.status]} {model.status.charAt(0).toUpperCase() + model.status.slice(1)}</span>
            {#if model.status === 'online' && model.capabilities?.includes('chat')}
              <div class="res-actions"><button class="btn-icon" onclick={() => navigateTo('chat')}>💬</button><button class="btn-icon">📋</button></div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<div class="section">
  <div class="section-header"><div class="section-title">🏠 Local Models</div></div>
  <div class="card" style="padding:6px 0">
    <div class="res-list">
      {#each localModels as model (model.id)}
        <div class="res-row">
          <div class="res-icon" style="background:rgba(245,158,11,.1)">🏠</div>
          <div class="res-info">
            <div class="res-name">{model.name}</div>
            <div class="res-meta">{model.runtime || 'Local'}</div>
          </div>
          <span class="badge {statusBadge[model.status]}">{statusIcon[model.status]} {model.status.charAt(0).toUpperCase() + model.status.slice(1)}</span>
          {#if model.capabilities?.includes('chat')}
            <div class="res-actions"><button class="btn-icon" onclick={() => navigateTo('chat')}>💬</button></div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>
