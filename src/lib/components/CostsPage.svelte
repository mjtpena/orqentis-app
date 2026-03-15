<script lang="ts">
  import { authStatus, activeEndpoint } from '../stores/auth';
</script>

<div class="page-header">
  <div class="page-title">Costs & Usage</div>
  <div class="page-subtitle">Estimated spend and token consumption across all surfaces</div>
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
    <p style="color:var(--text-3);font-size:.92rem">No Foundry endpoint discovered. Cost data requires a connected hub.</p>
  </div>
{:else}
  <div class="stats-row">
    <div class="stat"><div class="stat-val" style="color:var(--text-3)">—</div><div class="stat-label">This Month</div><div class="stat-sub">Cost Management API coming soon</div></div>
    <div class="stat"><div class="stat-val" style="color:var(--text-3)">—</div><div class="stat-label">Tokens Used</div><div class="stat-sub">Azure Monitor metrics</div></div>
    <div class="stat"><div class="stat-val" style="color:var(--text-3)">—</div><div class="stat-label">API Calls</div></div>
    <div class="stat"><div class="stat-val" style="color:var(--text-3)">—</div><div class="stat-label">Success Rate</div></div>
  </div>

  <div class="section">
    <div class="section-header"><div class="section-title">Cost Breakdown</div></div>
    <div class="card" style="padding:24px;text-align:center">
      <p style="color:var(--text-3);font-size:.88rem">Cost breakdown by model will be available after Azure Cost Management API integration.</p>
    </div>
  </div>
{/if}
