<script lang="ts">
  import { runs as mockRuns } from '../stores/data';
  import { activeEndpoint, authStatus } from '../stores/auth';
  import * as api from '../services/api';
  import type { FineTuningJob, BatchJob } from '../services/api';
  import type { Run } from '../types';

  let liveRuns = $state<Run[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let filter = $state('all');

  function formatDate(ts: number): string {
    const d = new Date(ts * 1000);
    const now = Date.now();
    const diff = now - d.getTime();
    if (diff < 3600_000) return `${Math.floor(diff / 60_000)} min ago`;
    if (diff < 86400_000) return `${Math.floor(diff / 3600_000)}h ago`;
    return `${Math.floor(diff / 86400_000)}d ago`;
  }

  function mapFineTuningJob(j: FineTuningJob): Run {
    return {
      id: j.id,
      name: `ft-${j.model}`,
      description: `Fine-Tuning · ${j.training_file} · ${formatDate(j.created_at)}`,
      type: 'fine_tuning',
      status: j.status === 'running' ? 'in_progress'
            : j.status === 'succeeded' ? 'succeeded'
            : j.status === 'failed' ? 'failed'
            : 'queued',
    };
  }

  function mapBatchJob(j: BatchJob): Run {
    return {
      id: j.id,
      name: `batch · ${j.endpoint}`,
      description: `Foundry Batch · ${j.completion_window} · ${formatDate(j.created_at)}`,
      type: 'batch',
      status: j.status === 'in_progress' ? 'in_progress'
            : j.status === 'completed' ? 'completed'
            : j.status === 'failed' ? 'failed'
            : 'queued',
    };
  }

  function loadRuns(endpoint: string) {
    loading = true;
    error = null;
    Promise.all([
      api.listFineTuningJobs(endpoint),
      api.listBatchJobs(endpoint),
    ])
      .then(([ftJobs, batchJobs]) => {
        liveRuns = [
          ...ftJobs.map(mapFineTuningJob),
          ...batchJobs.map(mapBatchJob),
        ].sort((a, b) => {
          const order: Record<string, number> = { in_progress: 0, queued: 1, succeeded: 2, completed: 2, failed: 3 };
          return (order[a.status] ?? 4) - (order[b.status] ?? 4);
        });
      })
      .catch(e => { error = e.toString(); })
      .finally(() => { loading = false; });
  }

  $effect(() => {
    const endpoint = $activeEndpoint;
    const status = $authStatus;
    if (status.signed_in && endpoint) {
      loadRuns(endpoint);
    } else {
      liveRuns = [];
    }
  });

  function retry() {
    const endpoint = $activeEndpoint;
    if (endpoint) loadRuns(endpoint);
  }

  let allRuns = $derived($authStatus.signed_in ? liveRuns : mockRuns);

  let filteredRuns = $derived(
    filter === 'all' ? allRuns
    : filter === 'active' ? allRuns.filter(r => ['in_progress', 'queued'].includes(r.status))
    : filter === 'completed' ? allRuns.filter(r => ['succeeded', 'completed'].includes(r.status))
    : filter === 'failed' ? allRuns.filter(r => r.status === 'failed')
    : allRuns
  );

  const statusBadge: Record<string, string> = { in_progress: 'badge-blue', queued: 'badge-yellow', succeeded: 'badge-green', completed: 'badge-green', failed: 'badge-red' };
  const statusIcon: Record<string, string> = { in_progress: '▶', queued: '⏰', succeeded: '✓', completed: '✓', failed: '✗' };
  const statusDot: Record<string, string> = { in_progress: 'dot-blue', queued: 'dot-yellow', succeeded: 'dot-green', completed: 'dot-green', failed: 'dot-red' };
</script>

<div class="page-header">
  <div class="page-title">Runs</div>
  <div class="page-subtitle">Agent executions, batch jobs, and fine-tuning across all platforms</div>
</div>

{#if !$authStatus.signed_in}
  <div class="not-connected-banner">
    <span>🔌</span>
    <span style="font-size:.85rem;color:var(--text-2)">Sign in to see your runs. Showing sample data.</span>
  </div>
{/if}

<div style="display:flex;gap:6px;margin-bottom:18px;flex-wrap:wrap">
  <button class="filter-chip" class:active={filter === 'all'} onclick={() => filter = 'all'}>All ({allRuns.length})</button>
  <button class="filter-chip" class:active={filter === 'active'} onclick={() => filter = 'active'}>▶️ Active ({allRuns.filter(r => ['in_progress', 'queued'].includes(r.status)).length})</button>
  <button class="filter-chip" class:active={filter === 'completed'} onclick={() => filter = 'completed'}>✓ Completed</button>
  <button class="filter-chip" class:active={filter === 'failed'} onclick={() => filter = 'failed'}>✗ Failed</button>
</div>

{#if loading}
  <div class="loading-state">
    <div class="spinner"></div>
    <p>Loading runs…</p>
  </div>
{:else if error}
  <div class="error-state">
    <p>⚠️ {error}</p>
    <button class="btn btn-outline" onclick={retry}>Retry</button>
  </div>
{:else if filteredRuns.length === 0}
  <div class="empty-state">
    <p>No runs found.</p>
  </div>
{:else}
  <div class="card" style="padding:6px 0">
    <div class="res-list">
      {#each filteredRuns as run (run.id)}
        <div class="res-row">
          <div class="dot {statusDot[run.status]}" style="margin:0 4px"></div>
          <div class="res-info">
            <div class="res-name">{run.name}</div>
            <div class="res-meta">{run.description}</div>
          </div>
          <span class="badge {statusBadge[run.status]}">{statusIcon[run.status]} {run.status.replace('_', ' ')}</span>
          {#if ['in_progress', 'queued'].includes(run.status)}
            <div class="res-actions" style="opacity:1"><button class="btn-icon">⏹</button></div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}
