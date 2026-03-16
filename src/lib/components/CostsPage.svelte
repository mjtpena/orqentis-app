<script lang="ts">
  import { authStatus, activeEndpoint, activeHub } from '../stores/auth';
  import * as api from '../services/api';
  import type { UsageMetrics } from '../services/api';

  let metrics: UsageMetrics | null = $state(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  function getAiServicesResourceId(): string | null {
    const hub = $activeHub;
    if (!hub) return null;
    const conn = hub.connections.find(
      (c) => c.properties.category === 'AIServices' || c.properties.category === 'AzureOpenAI'
    );
    return conn?.properties.metadata?.ResourceId ?? null;
  }

  async function loadMetrics() {
    const resourceId = getAiServicesResourceId();
    if (!resourceId) {
      error = 'No AI Services resource found on this hub.';
      return;
    }
    loading = true;
    error = null;
    try {
      metrics = await api.getUsageMetrics(resourceId);
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load usage metrics';
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if ($authStatus.signed_in && $activeEndpoint) {
      loadMetrics();
    }
  });

  function fmtNumber(n: number): string {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return Math.round(n).toLocaleString();
  }

  function fmtCost(n: number, currency: string): string {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(n);
  }

  function successRate(m: UsageMetrics): string {
    if (m.totalCalls === 0) return '—';
    return ((m.successfulCalls / m.totalCalls) * 100).toFixed(1) + '%';
  }

  // Compute max for bar chart scaling
  function maxDaily(m: UsageMetrics): number {
    return Math.max(1, ...m.daily.map(d => d.promptTokens + d.completionTokens));
  }

  function maxDailyCost(m: UsageMetrics): number {
    if (!m.cost) return 1;
    return Math.max(0.001, ...m.cost.daily.map(d => d.cost));
  }
</script>

<div class="page-header">
  <div class="page-title">Costs & Usage</div>
  <div class="page-subtitle">Token consumption and API usage over the last 30 days</div>
</div>

{#if !$authStatus.signed_in}
  <div class="not-connected-banner">
    <span>🔌</span>
    <span style="font-size:.85rem;color:var(--text-2)">Sign in to view cost and usage data.</span>
  </div>
  <div class="empty-state" style="padding:3rem">
    <p style="color:var(--text-3);font-size:.92rem">Cost and usage metrics will appear here once you're connected to Azure AI Foundry.</p>
  </div>
{:else if !$activeEndpoint}
  <div class="empty-state" style="padding:3rem">
    <p style="color:var(--text-3);font-size:.92rem">No Foundry endpoint discovered. Usage data requires a connected hub.</p>
  </div>
{:else if loading}
  <div class="empty-state" style="padding:3rem">
    <p style="color:var(--text-3);font-size:.92rem">Loading usage metrics…</p>
  </div>
{:else if error}
  <div class="not-connected-banner" style="border-color:var(--danger,#ef4444)">
    <span>⚠️</span>
    <span style="font-size:.85rem;color:var(--text-2)">{error}</span>
  </div>
  <div style="padding:1rem;text-align:center">
    <button class="btn-primary" onclick={loadMetrics}>Retry</button>
  </div>
{:else if metrics}
  <div class="stats-row">
    {#if metrics.cost}
      <div class="stat cost-stat">
        <div class="stat-val" style="color:#10b981">{fmtCost(metrics.cost.totalCost, metrics.cost.currency)}</div>
        <div class="stat-label">Total Cost</div>
        <div class="stat-sub">Last 30 days ({metrics.cost.currency})</div>
      </div>
    {/if}
    <div class="stat">
      <div class="stat-val">{fmtNumber(metrics.totalTokens)}</div>
      <div class="stat-label">Total Tokens</div>
      <div class="stat-sub">Last 30 days</div>
    </div>
    <div class="stat">
      <div class="stat-val">{fmtNumber(metrics.promptTokens)}</div>
      <div class="stat-label">Prompt Tokens</div>
      <div class="stat-sub">Input</div>
    </div>
    <div class="stat">
      <div class="stat-val">{fmtNumber(metrics.completionTokens)}</div>
      <div class="stat-label">Completion Tokens</div>
      <div class="stat-sub">Output</div>
    </div>
    <div class="stat">
      <div class="stat-val">{fmtNumber(metrics.totalCalls)}</div>
      <div class="stat-label">API Calls</div>
      <div class="stat-sub">Total requests</div>
    </div>
    <div class="stat">
      <div class="stat-val">{successRate(metrics)}</div>
      <div class="stat-label">Success Rate</div>
      <div class="stat-sub">{fmtNumber(metrics.successfulCalls)} successful</div>
    </div>
  </div>

  {#if metrics.cost && metrics.cost.daily.length > 0}
  <div class="section">
    <div class="section-header">
      <div class="section-title">Daily Cost ({metrics.cost.currency})</div>
    </div>
    <div class="card chart-card">
      <div class="bar-chart">
        {#each metrics.cost.daily as day}
          {@const max = maxDailyCost(metrics)}
          {@const pct = (day.cost / max) * 100}
          <div class="bar-col" title="{day.date}: {fmtCost(day.cost, metrics.cost?.currency ?? 'USD')}">
            <div class="bar-fill" style="height:{Math.max(pct, 1)}%">
              <div class="bar-cost" style="height:100%"></div>
            </div>
            <div class="bar-label">{day.date.slice(5)}</div>
          </div>
        {/each}
      </div>
    </div>
  </div>
  {/if}

  <div class="section">
    <div class="section-header">
      <div class="section-title">Daily Token Usage</div>
    </div>
    <div class="card chart-card">
      {#if metrics.daily.length === 0}
        <p style="color:var(--text-3);font-size:.88rem;text-align:center;padding:24px">No usage data for this period.</p>
      {:else}
        <div class="chart-legend">
          <span class="legend-item"><span class="legend-dot prompt"></span> Prompt</span>
          <span class="legend-item"><span class="legend-dot completion"></span> Completion</span>
        </div>
        <div class="bar-chart">
          {#each metrics.daily as day}
            {@const total = day.promptTokens + day.completionTokens}
            {@const max = maxDaily(metrics)}
            {@const pct = (total / max) * 100}
            {@const promptPct = total > 0 ? (day.promptTokens / total) * 100 : 0}
            <div class="bar-col" title="{day.date}: {fmtNumber(total)} tokens ({fmtNumber(day.totalCalls)} calls)">
              <div class="bar-fill" style="height:{Math.max(pct, 1)}%">
                <div class="bar-prompt" style="height:{promptPct}%"></div>
                <div class="bar-completion" style="height:{100 - promptPct}%"></div>
              </div>
              <div class="bar-label">{day.date.slice(5)}</div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <div class="section">
    <div class="section-header">
      <div class="section-title">Daily Breakdown</div>
    </div>
    <table class="data-table">
      <thead>
        <tr>
          <th>Date</th>
          {#if metrics.cost}<th>Cost</th>{/if}
          <th>Prompt Tokens</th>
          <th>Completion Tokens</th>
          <th>Total Tokens</th>
          <th>API Calls</th>
        </tr>
      </thead>
      <tbody>
        {#each [...metrics.daily].reverse() as day}
          {@const costDay = metrics.cost?.daily.find(d => d.date === day.date)}
          <tr>
            <td>{day.date}</td>
            {#if metrics.cost}<td style="color:#10b981;font-weight:500">{costDay ? fmtCost(costDay.cost, metrics.cost.currency) : '—'}</td>{/if}
            <td>{fmtNumber(day.promptTokens)}</td>
            <td>{fmtNumber(day.completionTokens)}</td>
            <td>{fmtNumber(day.promptTokens + day.completionTokens)}</td>
            <td>{fmtNumber(day.totalCalls)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .cost-stat {
    border-left: 3px solid #10b981;
  }
  .chart-card {
    padding: 20px;
  }
  .chart-legend {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
    font-size: .82rem;
    color: var(--text-2);
  }
  .legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .legend-dot {
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }
  .legend-dot.prompt { background: #6366f1; }
  .legend-dot.completion { background: #22d3ee; }

  .bar-chart {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 180px;
    padding-top: 8px;
  }
  .bar-col {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
    justify-content: flex-end;
    min-width: 0;
    cursor: default;
  }
  .bar-fill {
    width: 100%;
    max-width: 24px;
    border-radius: 3px 3px 0 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .bar-prompt { background: #6366f1; }
  .bar-completion { background: #22d3ee; }
  .bar-cost { background: #10b981; }
  .bar-label {
    font-size: .62rem;
    color: var(--text-3);
    margin-top: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: .85rem;
  }
  .data-table th {
    text-align: left;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    color: var(--text-3);
    font-weight: 500;
    font-size: .78rem;
    text-transform: uppercase;
    letter-spacing: .04em;
  }
  .data-table td {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    color: var(--text-1);
  }
  .data-table tr:hover td {
    background: var(--bg-2);
  }

  .btn-primary {
    padding: 6px 16px;
    background: var(--accent, #6366f1);
    color: #fff;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: .85rem;
  }
  .btn-primary:hover { opacity: 0.9; }
</style>
