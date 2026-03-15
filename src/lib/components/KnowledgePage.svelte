<script lang="ts">
  import { knowledge } from '../stores/data';

  const vectorStores = knowledge.filter(k => k.type === 'vector_store');
  const files = knowledge.filter(k => k.type === 'file');
  const connected = knowledge.filter(k => ['sharepoint', 'onedrive', 'local_dir'].includes(k.type));

  const typeIcon: Record<string, string> = { vector_store: '🗃', file: '📄', sharepoint: '📂', onedrive: '☁️', local_dir: '📁' };
  const typeColor: Record<string, string> = { vector_store: 'rgba(168,85,247,.1)', file: 'rgba(28,194,188,.1)', sharepoint: 'rgba(26,137,240,.1)', onedrive: 'rgba(26,137,240,.1)', local_dir: 'rgba(245,158,11,.1)' };
</script>

<div class="page-header">
  <div style="display:flex;align-items:center;justify-content:space-between">
    <div>
      <div class="page-title">Knowledge</div>
      <div class="page-subtitle">Files, vector stores, and connected data sources powering your agents</div>
    </div>
    <button class="btn btn-primary">📤 Upload</button>
  </div>
</div>

{#if vectorStores.length}
<div class="section">
  <div class="section-header"><div class="section-title">Vector Stores</div></div>
  <div class="card" style="padding:6px 0">
    <div class="res-list">
      {#each vectorStores as k (k.id)}
        <div class="res-row">
          <div class="res-icon" style="background:{typeColor[k.type]}">{typeIcon[k.type]}</div>
          <div class="res-info">
            <div class="res-name">{k.name}</div>
            <div class="res-meta">{k.fileCount} files · {k.size}{k.usedBy?.length ? ` · Used by ${k.usedBy.join(', ')}` : ''}</div>
          </div>
          <span class="badge badge-green">● Ready</span>
        </div>
      {/each}
    </div>
  </div>
</div>
{/if}

{#if files.length}
<div class="section">
  <div class="section-header"><div class="section-title">Project Files</div></div>
  <div class="card" style="padding:6px 0">
    <div class="res-list">
      {#each files as k (k.id)}
        <div class="res-row">
          <div class="res-icon" style="background:{typeColor[k.type]}">{typeIcon[k.type]}</div>
          <div class="res-info">
            <div class="res-name">{k.name}</div>
            <div class="res-meta">{k.size} · {k.purpose} · {k.updatedAt}</div>
          </div>
          <span class="badge badge-muted">{k.purpose}</span>
        </div>
      {/each}
    </div>
  </div>
</div>
{/if}

{#if connected.length}
<div class="section">
  <div class="section-header"><div class="section-title">Connected Data</div></div>
  <div class="card" style="padding:6px 0">
    <div class="res-list">
      {#each connected as k (k.id)}
        <div class="res-row">
          <div class="res-icon" style="background:{typeColor[k.type]}">{typeIcon[k.type]}</div>
          <div class="res-info">
            <div class="res-name">{k.name}</div>
            <div class="res-meta">{k.fileCount ? `${k.fileCount} documents · ` : ''}{k.updatedAt}{k.usedBy?.length ? ` · ${k.usedBy.join(', ')}` : ''}</div>
          </div>
          <span class="badge badge-green">● {k.status === 'connected' ? 'Connected' : 'Synced'}</span>
        </div>
      {/each}
    </div>
  </div>
</div>
{/if}
