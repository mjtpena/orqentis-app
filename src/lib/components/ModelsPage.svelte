<script lang="ts">
  import { navigateTo } from '../stores/navigation';
  import { authStatus, armDeployments, localAgents, activeHub } from '../stores/auth';
  import type { ArmDeployment } from '../services/api';
  import type { Model } from '../types';

  function mapDeployment(d: ArmDeployment): Model {
    return {
      id: d.name,
      name: d.properties.model?.name ?? d.name,
      version: d.properties.model?.version,
      sku: d.sku?.name,
      source: 'cloud',
      status: d.properties.provisioning_state === 'Succeeded' ? 'online'
            : d.properties.provisioning_state === 'Failed' ? 'failed'
            : 'provisioning',
      capabilities: ['chat'],
    };
  }

  let cloudModels = $derived($armDeployments.map(mapDeployment));

  let localModels = $derived($localAgents.map(a => ({
    id: a.id,
    name: a.name,
    runtime: a.runtime,
    source: 'local' as const,
    status: a.status === 'running' ? 'online' as const : 'offline' as const,
    capabilities: ['chat'],
    endpoint: a.endpoint,
  })));

  let projects = $derived($activeHub?.projects ?? []);

  const statusBadge: Record<string, string> = { online: 'badge-green', provisioning: 'badge-yellow', failed: 'badge-red', offline: 'badge-muted' };
  const statusIcon: Record<string, string> = { online: '●', provisioning: '⟳', failed: '✗', offline: '○' };
</script>

<div class="page-header">
  <div style="display:flex;align-items:center;justify-content:space-between">
    <div>
      <div class="page-title">Models</div>
      <div class="page-subtitle">Cloud deployments, local model runtimes, and Foundry projects</div>
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

<!-- Foundry Projects -->
{#if projects.length > 0}
<div class="section">
  <div class="section-header"><div class="section-title">📁 Foundry Projects</div></div>
  <div class="card" style="padding:6px 0">
    <div class="res-list">
      {#each projects as proj (proj.id)}
        <div class="res-row">
          <div class="res-icon" style="background:rgba(99,102,241,.1)">📁</div>
          <div class="res-info">
            <div class="res-name">{proj.name}</div>
            <div class="res-meta">{proj.location} · {proj.kind}</div>
          </div>
          <span class="badge badge-blue">Project</span>
        </div>
      {/each}
    </div>
  </div>
</div>
{/if}

<!-- Cloud deployments -->
<div class="section">
  <div class="section-header"><div class="section-title">☁️ Azure AI Foundry</div></div>
  {#if !$authStatus.signed_in}
    <div class="empty-state">
      <p>Sign in to see your cloud deployments.</p>
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

<!-- Local models -->
{#if localModels.length > 0}
<div class="section">
  <div class="section-header"><div class="section-title">🏠 Local Runtimes</div></div>
  <div class="card" style="padding:6px 0">
    <div class="res-list">
      {#each localModels as model (model.id)}
        <div class="res-row">
          <div class="res-icon" style="background:rgba(245,158,11,.1)">🏠</div>
          <div class="res-info">
            <div class="res-name">{model.name}</div>
            <div class="res-meta">{model.runtime} · {model.endpoint}</div>
          </div>
          <span class="badge {statusBadge[model.status]}">{statusIcon[model.status]} {model.status.charAt(0).toUpperCase() + model.status.slice(1)}</span>
          <div class="res-actions"><button class="btn-icon" onclick={() => navigateTo('chat')}>💬</button></div>
        </div>
      {/each}
    </div>
  </div>
</div>
{/if}
