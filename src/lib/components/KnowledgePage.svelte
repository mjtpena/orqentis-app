<script lang="ts">
  import { knowledge as mockKnowledge } from '../stores/data';
  import { activeEndpoint, authStatus } from '../stores/auth';
  import * as api from '../services/api';
  import type { VectorStore, FoundryFile } from '../services/api';
  import type { KnowledgeSource } from '../types';

  let liveVectorStores = $state<KnowledgeSource[]>([]);
  let liveFiles = $state<KnowledgeSource[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  const mockVectorStores = mockKnowledge.filter(k => k.type === 'vector_store');
  const mockFiles = mockKnowledge.filter(k => k.type === 'file');
  const mockConnected = mockKnowledge.filter(k => ['sharepoint', 'onedrive', 'local_dir'].includes(k.type));

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(ts: number): string {
    return new Date(ts * 1000).toLocaleDateString();
  }

  function mapVectorStore(vs: VectorStore): KnowledgeSource {
    return {
      id: vs.id,
      name: vs.name || vs.id,
      type: 'vector_store',
      fileCount: vs.file_counts.total,
      size: formatBytes(vs.usage_bytes),
      status: vs.status === 'completed' ? 'ready' : 'syncing',
    };
  }

  function mapFile(f: FoundryFile): KnowledgeSource {
    return {
      id: f.id,
      name: f.filename,
      type: 'file',
      size: formatBytes(f.bytes),
      purpose: f.purpose,
      status: f.status === 'processed' ? 'ready' : 'syncing',
      updatedAt: formatDate(f.created_at),
    };
  }

  function loadKnowledge(endpoint: string) {
    loading = true;
    error = null;
    Promise.all([
      api.listVectorStores(endpoint),
      api.listFiles(endpoint),
    ])
      .then(([vs, files]) => {
        liveVectorStores = vs.map(mapVectorStore);
        liveFiles = files.map(mapFile);
      })
      .catch(e => { error = e.toString(); })
      .finally(() => { loading = false; });
  }

  $effect(() => {
    const endpoint = $activeEndpoint;
    const status = $authStatus;
    if (status.signed_in && endpoint) {
      loadKnowledge(endpoint);
    } else {
      liveVectorStores = [];
      liveFiles = [];
    }
  });

  function retry() {
    const endpoint = $activeEndpoint;
    if (endpoint) loadKnowledge(endpoint);
  }

  let vectorStores = $derived($authStatus.signed_in ? liveVectorStores : mockVectorStores);
  let files = $derived($authStatus.signed_in ? liveFiles : mockFiles);
  let connected = $derived(mockConnected);

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

{#if !$authStatus.signed_in}
  <div class="not-connected-banner">
    <span>🔌</span>
    <span style="font-size:.85rem;color:var(--text-2)">Sign in to see your Foundry knowledge. Showing sample data.</span>
  </div>
{/if}

{#if loading}
  <div class="loading-state">
    <div class="spinner"></div>
    <p>Loading knowledge…</p>
  </div>
{:else if error}
  <div class="error-state">
    <p>⚠️ {error}</p>
    <button class="btn btn-outline" onclick={retry}>Retry</button>
  </div>
{:else}
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
            <span class="badge badge-green">● {k.status === 'ready' ? 'Ready' : 'Syncing'}</span>
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
              <div class="res-meta">{k.size} · {k.purpose}{k.updatedAt ? ` · ${k.updatedAt}` : ''}</div>
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

  {#if vectorStores.length === 0 && files.length === 0 && connected.length === 0}
    <div class="empty-state">
      <p>No knowledge sources found.</p>
    </div>
  {/if}
{/if}
