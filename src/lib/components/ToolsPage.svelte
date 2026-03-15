<script lang="ts">
  import { tools } from '../stores/data';

  const typeLabel: Record<string, string> = { builtin: 'Built-in', azure: 'Azure', openapi: 'OpenAPI', m365: 'M365' };
</script>

<div class="page-header">
  <div class="page-title">Tools</div>
  <div class="page-subtitle">Capabilities available to your agents — built-in, Azure, and custom</div>
</div>

<div class="agent-grid">
  {#each tools as tool (tool.id)}
    <div class="card">
      <div style="display:flex;align-items:center;gap:12px;margin-bottom:10px">
        <div style="font-size:1.5rem">
          {tool.type === 'builtin' ? (tool.name.includes('Search') ? '🔍' : '💻') : tool.type === 'azure' ? '🔎' : tool.type === 'openapi' ? '⚡' : '📊'}
        </div>
        <div>
          <div style="font-weight:600;font-size:.92rem">{tool.name}</div>
          <div style="font-size:.72rem;color:var(--text-3)">{typeLabel[tool.type]}</div>
        </div>
      </div>
      <div style="font-size:.82rem;color:var(--text-2);margin-bottom:10px">{tool.description}</div>
      {#if tool.usedBy?.length}
        <div style="font-size:.72rem;color:var(--text-3)">Used by: {tool.usedBy.join(', ')}</div>
      {/if}
    </div>
  {/each}
</div>
