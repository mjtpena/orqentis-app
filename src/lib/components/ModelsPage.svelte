<script lang="ts">
  import { models } from '../stores/data';
  import { navigateTo } from '../stores/navigation';

  const cloudModels = models.filter(m => m.source === 'cloud');
  const localModels = models.filter(m => m.source === 'local');

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

<div class="section">
  <div class="section-header"><div class="section-title">☁️ Azure AI Foundry</div></div>
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
