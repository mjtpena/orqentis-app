<script lang="ts">
  import { activeEndpoint, authStatus } from '../stores/auth';
  import * as api from '../services/api';
  import type { Tool } from '../types';

  let tools = $state<Tool[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  // Extract unique tools from agents
  function loadTools(endpoint: string) {
    loading = true;
    error = null;
    api.listAgents(endpoint)
      .then(agents => {
        const toolMap = new Map<string, Tool>();
        for (const agent of agents) {
          for (const t of agent.tools) {
            if (!toolMap.has(t.type)) {
              const label = t.type.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
              toolMap.set(t.type, {
                id: t.type,
                name: label,
                description: `${t.type} tool capability`,
                type: t.type === 'code_interpreter' || t.type === 'file_search' ? 'builtin' : 'azure',
                usedBy: [],
              });
            }
            const tool = toolMap.get(t.type)!;
            const agentName = agent.name || agent.id;
            if (!tool.usedBy?.includes(agentName)) {
              tool.usedBy = [...(tool.usedBy || []), agentName];
            }
          }
        }
        tools = Array.from(toolMap.values());
      })
      .catch(e => { error = e.toString(); })
      .finally(() => { loading = false; });
  }

  $effect(() => {
    const endpoint = $activeEndpoint;
    const status = $authStatus;
    if (status.signed_in && endpoint) {
      loadTools(endpoint);
    } else {
      tools = [];
    }
  });

  const typeLabel: Record<string, string> = { builtin: 'Built-in', azure: 'Azure', openapi: 'OpenAPI', m365: 'M365' };
</script>

<div class="page-header">
  <div class="page-title">Tools</div>
  <div class="page-subtitle">Capabilities available to your agents — built-in, Azure, and custom</div>
</div>

{#if !$authStatus.signed_in}
  <div class="not-connected-banner">
    <span>🔌</span>
    <span style="font-size:.85rem;color:var(--text-2)">Sign in to see tools used by your agents.</span>
  </div>
{/if}

{#if loading}
  <div class="loading-state">
    <div class="spinner"></div>
    <p>Loading tools…</p>
  </div>
{:else if error}
  <div class="error-state">
    <p>⚠️ {error}</p>
  </div>
{:else if tools.length === 0}
  <div class="empty-state">
    <p style="color:var(--text-3);font-size:.88rem">{$authStatus.signed_in ? 'No agent tools found.' : 'Sign in to discover tools.'}</p>
  </div>
{:else}
  <div class="agent-grid">
    {#each tools as tool (tool.id)}
      <div class="card">
        <div style="display:flex;align-items:center;gap:12px;margin-bottom:10px">
          <div style="font-size:1.5rem">
            {tool.type === 'builtin' ? (tool.name.includes('Search') ? '🔍' : '💻') : tool.type === 'azure' ? '🔎' : tool.type === 'openapi' ? '⚡' : '📊'}
          </div>
          <div>
            <div style="font-weight:600;font-size:.92rem">{tool.name}</div>
            <div style="font-size:.72rem;color:var(--text-3)">{typeLabel[tool.type] || tool.type}</div>
          </div>
        </div>
        <div style="font-size:.82rem;color:var(--text-2);margin-bottom:10px">{tool.description}</div>
        {#if tool.usedBy?.length}
          <div style="font-size:.72rem;color:var(--text-3)">Used by: {tool.usedBy.join(', ')}</div>
        {/if}
      </div>
    {/each}
  </div>
{/if}
