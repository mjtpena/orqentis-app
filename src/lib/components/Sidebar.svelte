<script lang="ts">
  import { currentPage, navigateTo, toggleTheme, theme } from '../stores/navigation';
  import { authStatus, authLoading, discoveryLoading, hubs } from '../stores/auth';
  import type { Page } from '../types';

  const navGroups: { label?: string; items: { id: Page; icon: string; name: string }[] }[] = [
    {
      items: [{ id: 'home', icon: '🏠', name: 'Home' }]
    },
    {
      label: 'Operate',
      items: [
        { id: 'agents', icon: '🤖', name: 'Agents' },
        { id: 'models', icon: '🧠', name: 'Models' },
        { id: 'chat', icon: '💬', name: 'Chat' },
        { id: 'runs', icon: '▶️', name: 'Runs' },
      ]
    },
    {
      label: 'Compose',
      items: [
        { id: 'knowledge', icon: '📚', name: 'Knowledge' },
        { id: 'tools', icon: '🔧', name: 'Tools' },
        { id: 'connections', icon: '🔗', name: 'Connections' },
      ]
    },
    {
      label: 'Observe',
      items: [
        { id: 'costs', icon: '💰', name: 'Costs' },
        { id: 'trust', icon: '🛡️', name: 'Trust Center' },
      ]
    }
  ];
</script>

<nav class="sidebar">
  <div class="sidebar-brand">
    <div class="logo">O</div>
    <div>
      <div class="name">Orqentis</div>
      <div class="tag">Agent Control Plane</div>
    </div>
  </div>

  <div class="sidebar-nav">
    {#each navGroups as group}
      <div class="nav-group">
        {#if group.label}
          <div class="nav-group-label">{group.label}</div>
        {/if}
        {#each group.items as item}
          <button
            class="nav-item"
            class:active={$currentPage === item.id}
            onclick={() => navigateTo(item.id)}
          >
            <span class="nav-icon">{item.icon}</span>
            {item.name}
          </button>
        {/each}
      </div>
    {/each}
  </div>

  <div class="sidebar-footer">
    {#if $authStatus.signed_in}
      <div class="avatar">{$authStatus.user_name ? $authStatus.user_name.charAt(0).toUpperCase() : '?'}</div>
      <div class="user-info">
        <div class="user-name">{$authStatus.user_name ?? 'User'}</div>
        <div class="user-org">{$authStatus.auth_mode === 'az_cli' ? 'Azure CLI' : $authStatus.auth_mode ?? ''}{$authStatus.tenant_id ? ` · ${$authStatus.tenant_id.slice(0,8)}…` : ''}</div>
      </div>
    {:else}
      <div class="avatar" style="background:var(--bg-3)">?</div>
      <div class="user-info">
        <div class="user-name" style="color:var(--text-3)">Not signed in</div>
      </div>
    {/if}
    <button class="btn-icon" onclick={toggleTheme} title="Toggle theme">
      {$theme === 'dark' ? '🌙' : '☀️'}
    </button>
  </div>

  <div class="sidebar-connection">
    {#if $authLoading || $discoveryLoading}
      <span class="conn-dot conn-dot--loading"></span>
      <span class="conn-label">Connecting…</span>
    {:else if $authStatus.signed_in}
      <span class="conn-dot conn-dot--connected"></span>
      <span class="conn-label">{$hubs.length} hub{$hubs.length !== 1 ? 's' : ''}</span>
      <span class="conn-badge">{$authStatus.auth_mode === 'az_cli' ? 'CLI' : $authStatus.auth_mode === 'oauth' ? 'OAuth' : $authStatus.auth_mode}</span>
    {:else}
      <span class="conn-dot conn-dot--disconnected"></span>
      <span class="conn-label">Not connected</span>
    {/if}
  </div>
</nav>

<style>
  .sidebar {
    width: var(--sidebar-w); flex-shrink: 0; background: var(--bg-1);
    border-right: 1px solid var(--border); display: flex; flex-direction: column;
    overflow: hidden; z-index: 10;
  }
  .sidebar-brand {
    padding: 20px 18px 16px; display: flex; align-items: center; gap: 10px;
    border-bottom: 1px solid var(--border);
  }
  .logo {
    width: 28px; height: 28px; border-radius: var(--radius-sm); background: var(--gradient);
    display: flex; align-items: center; justify-content: center;
    font-size: .75rem; font-weight: 800; color: #fff;
  }
  .name { font-weight: 700; font-size: 1.05rem; letter-spacing: -.01em; }
  .tag { font-size: .62rem; color: var(--text-3); font-weight: 500; text-transform: uppercase; letter-spacing: .04em; }
  .sidebar-nav { flex: 1; overflow-y: auto; padding: 12px 10px; }
  .nav-group { margin-bottom: 18px; }
  .nav-group-label {
    font-size: .65rem; font-weight: 600; text-transform: uppercase;
    letter-spacing: .06em; color: var(--text-3); padding: 4px 10px 6px; user-select: none;
  }
  .nav-item {
    display: flex; align-items: center; gap: 10px; padding: 8px 10px;
    border-radius: var(--radius-sm); cursor: pointer; transition: all var(--transition);
    font-size: .88rem; color: var(--text-2); position: relative; width: 100%; text-align: left;
  }
  .nav-item:hover { background: var(--bg-hover); color: var(--text-1); }
  .nav-item.active { background: var(--bg-selected); color: var(--brand); font-weight: 600; }
  .nav-item.active::before {
    content: ''; position: absolute; left: 0; top: 6px; bottom: 6px;
    width: 3px; border-radius: 2px; background: var(--brand);
  }
  .nav-icon { width: 20px; text-align: center; font-size: .95rem; flex-shrink: 0; }
  .sidebar-footer {
    padding: 12px 14px; border-top: 1px solid var(--border);
    display: flex; align-items: center; gap: 10px;
  }
  .avatar {
    width: 30px; height: 30px; border-radius: 50%; background: var(--gradient);
    display: flex; align-items: center; justify-content: center;
    font-size: .68rem; font-weight: 700; color: #fff;
  }
  .user-info { flex: 1; min-width: 0; }
  .user-name { font-size: .82rem; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .user-org { font-size: .68rem; color: var(--text-3); }
  .sidebar-connection {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    border-top: 1px solid var(--border);
    font-size: .7rem;
    color: var(--text-3);
  }
  .conn-dot {
    width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0;
  }
  .conn-dot--connected { background: #34d399; }
  .conn-dot--loading { background: #fbbf24; animation: conn-pulse 1s ease-in-out infinite; }
  .conn-dot--disconnected { background: var(--text-3); opacity: .4; }
  @keyframes conn-pulse {
    0%, 100% { opacity: .4; }
    50% { opacity: 1; }
  }
  .conn-label { flex: 1; }
  .conn-badge {
    font-size: .6rem; font-weight: 600; text-transform: uppercase;
    padding: 1px 6px; border-radius: 99px;
    background: rgba(99,102,241,.12); color: var(--brand);
    letter-spacing: .03em;
  }
</style>
