<script lang="ts">
  import { runs } from '../stores/data';

  const statusBadge: Record<string, string> = { in_progress: 'badge-blue', queued: 'badge-yellow', succeeded: 'badge-green', completed: 'badge-green', failed: 'badge-red' };
  const statusIcon: Record<string, string> = { in_progress: '▶', queued: '⏰', succeeded: '✓', completed: '✓', failed: '✗' };
  const statusDot: Record<string, string> = { in_progress: 'dot-blue', queued: 'dot-yellow', succeeded: 'dot-green', completed: 'dot-green', failed: 'dot-red' };
</script>

<div class="page-header">
  <div class="page-title">Runs</div>
  <div class="page-subtitle">Agent executions, batch jobs, and fine-tuning across all platforms</div>
</div>

<div style="display:flex;gap:6px;margin-bottom:18px;flex-wrap:wrap">
  <button class="filter-chip active">All</button>
  <button class="filter-chip">▶️ Active ({runs.filter(r => ['in_progress', 'queued'].includes(r.status)).length})</button>
  <button class="filter-chip">✓ Completed</button>
  <button class="filter-chip">✗ Failed</button>
</div>

<div class="card" style="padding:6px 0">
  <div class="res-list">
    {#each runs as run (run.id)}
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
