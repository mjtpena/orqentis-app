<script lang="ts">
  import { currentPage, navigateTo } from '../stores/navigation';
  import { authStatus, hubs, studioAgents, m365Agents, localAgents, armDeployments } from '../stores/auth';
  import type { Page } from '../types';

  let { open = $bindable(false) } = $props();
  let query = $state('');
  let selectedIndex = $state(0);
  let inputEl = $state<HTMLInputElement>(null!);

  interface CommandItem {
    id: string;
    label: string;
    description?: string;
    icon: string;
    category: string;
    action: () => void;
  }

  const pageCommands: CommandItem[] = [
    { id: 'nav-home', label: 'Home', description: 'Dashboard overview', icon: '🏠', category: 'Navigation', action: () => navigateTo('home') },
    { id: 'nav-agents', label: 'Agents', description: 'Manage AI agents', icon: '🤖', category: 'Navigation', action: () => navigateTo('agents') },
    { id: 'nav-models', label: 'Models', description: 'Cloud & local models', icon: '🧠', category: 'Navigation', action: () => navigateTo('models') },
    { id: 'nav-chat', label: 'Chat', description: 'Start a conversation', icon: '💬', category: 'Navigation', action: () => navigateTo('chat') },
    { id: 'nav-runs', label: 'Runs', description: 'Fine-tuning & batch jobs', icon: '▶️', category: 'Navigation', action: () => navigateTo('runs') },
    { id: 'nav-knowledge', label: 'Knowledge', description: 'Vector stores & files', icon: '📚', category: 'Navigation', action: () => navigateTo('knowledge') },
    { id: 'nav-tools', label: 'Tools', description: 'Agent tools', icon: '🔧', category: 'Navigation', action: () => navigateTo('tools') },
    { id: 'nav-connections', label: 'Connections', description: 'Azure, M365, local', icon: '🔗', category: 'Navigation', action: () => navigateTo('connections') },
    { id: 'nav-costs', label: 'Costs & Usage', description: 'Token consumption & metrics', icon: '💰', category: 'Navigation', action: () => navigateTo('costs') },
    { id: 'nav-trust', label: 'Trust Center', description: 'Privacy & security', icon: '🛡️', category: 'Navigation', action: () => navigateTo('trust') },
  ];

  const actionCommands: CommandItem[] = [
    { id: 'act-new-chat', label: 'New Chat', description: 'Start a new chat session', icon: '✨', category: 'Actions', action: () => navigateTo('chat') },
  ];

  let dynamicCommands = $derived.by(() => {
    const cmds: CommandItem[] = [];

    // Deployments as quick-chat targets
    for (const d of $armDeployments) {
      const modelName = d.properties.model?.name ?? d.name;
      cmds.push({
        id: `deploy-${d.name}`,
        label: `Chat with ${modelName}`,
        description: `${d.sku?.name ?? 'Deployment'} · Foundry`,
        icon: '🧠',
        category: 'Deployments',
        action: () => navigateTo('chat'),
      });
    }

    // Foundry hub names
    for (const h of $hubs) {
      cmds.push({
        id: `hub-${h.workspace.id}`,
        label: h.workspace.name,
        description: `Hub · ${h.workspace.location}`,
        icon: '🏗️',
        category: 'Hubs',
        action: () => navigateTo('models'),
      });
    }

    // Studio agents
    for (const a of $studioAgents) {
      cmds.push({
        id: `studio-${a.id}`,
        label: a.name,
        description: a.description ?? 'Copilot Studio bot',
        icon: '🟣',
        category: 'Studio Agents',
        action: () => navigateTo('agents'),
      });
    }

    // M365 agents
    for (const a of $m365Agents) {
      cmds.push({
        id: `m365-${a.id}`,
        label: a.name,
        description: a.description ?? 'M365 agent',
        icon: '🔷',
        category: 'M365 Agents',
        action: () => navigateTo('agents'),
      });
    }

    // Local agents
    for (const a of $localAgents) {
      cmds.push({
        id: `local-${a.id}`,
        label: a.name,
        description: `${a.runtime} · ${a.model ?? 'Local'}`,
        icon: '🏠',
        category: 'Local Agents',
        action: () => navigateTo('agents'),
      });
    }

    return cmds;
  });

  let allCommands = $derived([...pageCommands, ...actionCommands, ...dynamicCommands]);

  let filtered = $derived.by(() => {
    if (!query.trim()) return allCommands;
    const q = query.toLowerCase();
    return allCommands.filter(
      (c) =>
        c.label.toLowerCase().includes(q) ||
        (c.description?.toLowerCase().includes(q) ?? false) ||
        c.category.toLowerCase().includes(q)
    );
  });

  // Group by category
  let grouped = $derived.by(() => {
    const map = new Map<string, CommandItem[]>();
    for (const item of filtered) {
      const group = map.get(item.category) ?? [];
      group.push(item);
      map.set(item.category, group);
    }
    return map;
  });

  $effect(() => {
    if (open) {
      query = '';
      selectedIndex = 0;
      requestAnimationFrame(() => inputEl?.focus());
    }
  });

  // Reset selection when filter changes
  $effect(() => {
    filtered; // subscribe
    selectedIndex = 0;
  });

  function close() {
    open = false;
  }

  function selectItem(item: CommandItem) {
    item.action();
    close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      close();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (filtered[selectedIndex]) selectItem(filtered[selectedIndex]);
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('cmd-backdrop')) {
      close();
    }
  }

  // Flat index tracker for keyboard nav
  function flatIndex(category: string, localIdx: number): number {
    let offset = 0;
    for (const [cat, items] of grouped) {
      if (cat === category) return offset + localIdx;
      offset += items.length;
    }
    return 0;
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="cmd-backdrop" onclick={handleBackdropClick}>
    <div class="cmd-dialog" role="dialog" aria-label="Command palette">
      <div class="cmd-input-wrapper">
        <span class="cmd-search-icon">⌘</span>
        <input
          bind:this={inputEl}
          bind:value={query}
          class="cmd-input"
          placeholder="Type a command or search…"
          onkeydown={handleKeydown}
          spellcheck="false"
          autocomplete="off"
        />
        <kbd class="cmd-kbd">ESC</kbd>
      </div>

      <div class="cmd-list">
        {#if filtered.length === 0}
          <div class="cmd-empty">No results found for "{query}"</div>
        {:else}
          {#each [...grouped] as [category, items] (category)}
            <div class="cmd-group-label">{category}</div>
            {#each items as item, i (item.id)}
              {@const idx = flatIndex(category, i)}
              <button
                class="cmd-item"
                class:cmd-item-selected={idx === selectedIndex}
                onclick={() => selectItem(item)}
                onmouseenter={() => selectedIndex = idx}
              >
                <span class="cmd-item-icon">{item.icon}</span>
                <div class="cmd-item-info">
                  <span class="cmd-item-label">{item.label}</span>
                  {#if item.description}
                    <span class="cmd-item-desc">{item.description}</span>
                  {/if}
                </div>
                <span class="cmd-item-shortcut">↵</span>
              </button>
            {/each}
          {/each}
        {/if}
      </div>

      <div class="cmd-footer">
        <span><kbd>↑↓</kbd> navigate</span>
        <span><kbd>↵</kbd> select</span>
        <span><kbd>esc</kbd> close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .cmd-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    z-index: 9999;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
  }
  .cmd-dialog {
    width: 560px;
    max-height: 480px;
    background: var(--bg-1);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: cmd-slide-in 0.15s ease-out;
  }
  @keyframes cmd-slide-in {
    from { opacity: 0; transform: translateY(-8px) scale(0.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }
  .cmd-input-wrapper {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
  }
  .cmd-search-icon {
    font-size: 0.9rem;
    color: var(--text-3);
    flex-shrink: 0;
    width: 20px;
    text-align: center;
  }
  .cmd-input {
    flex: 1;
    font-size: 0.95rem;
    border: none;
    background: transparent;
    color: var(--text-1);
    outline: none;
  }
  .cmd-input::placeholder {
    color: var(--text-3);
  }
  .cmd-kbd {
    font-size: 0.6rem;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--bg-3);
    border: 1px solid var(--border);
    color: var(--text-3);
    font-family: inherit;
    line-height: 1;
  }
  .cmd-list {
    flex: 1;
    overflow-y: auto;
    padding: 6px;
  }
  .cmd-group-label {
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-3);
    padding: 8px 10px 4px;
  }
  .cmd-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 0.08s;
    text-align: left;
    border: none;
    background: transparent;
    color: inherit;
  }
  .cmd-item:hover,
  .cmd-item-selected {
    background: var(--bg-hover);
  }
  .cmd-item-selected {
    background: var(--bg-selected);
  }
  .cmd-item-icon {
    font-size: 0.92rem;
    width: 24px;
    text-align: center;
    flex-shrink: 0;
  }
  .cmd-item-info {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .cmd-item-label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-1);
  }
  .cmd-item-desc {
    font-size: 0.72rem;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .cmd-item-shortcut {
    font-size: 0.72rem;
    color: var(--text-3);
    opacity: 0;
    transition: opacity 0.08s;
  }
  .cmd-item-selected .cmd-item-shortcut,
  .cmd-item:hover .cmd-item-shortcut {
    opacity: 1;
  }
  .cmd-empty {
    padding: 24px;
    text-align: center;
    color: var(--text-3);
    font-size: 0.85rem;
  }
  .cmd-footer {
    display: flex;
    gap: 16px;
    padding: 8px 16px;
    border-top: 1px solid var(--border);
    font-size: 0.65rem;
    color: var(--text-3);
  }
  .cmd-footer kbd {
    font-size: 0.6rem;
    font-weight: 600;
    padding: 1px 4px;
    border-radius: 3px;
    background: var(--bg-3);
    border: 1px solid var(--border);
    font-family: inherit;
    margin-right: 2px;
  }
</style>
