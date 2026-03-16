<script lang="ts">
  import { onMount } from 'svelte';
  import './app.css';
  import Sidebar from './lib/components/Sidebar.svelte';
  import HomePage from './lib/components/HomePage.svelte';
  import AgentsPage from './lib/components/AgentsPage.svelte';
  import ModelsPage from './lib/components/ModelsPage.svelte';
  import ChatPage from './lib/components/ChatPage.svelte';
  import RunsPage from './lib/components/RunsPage.svelte';
  import KnowledgePage from './lib/components/KnowledgePage.svelte';
  import ToolsPage from './lib/components/ToolsPage.svelte';
  import ConnectionsPage from './lib/components/ConnectionsPage.svelte';
  import CostsPage from './lib/components/CostsPage.svelte';
  import TrustPage from './lib/components/TrustPage.svelte';
  import { currentPage, navigateTo, theme, toggleTheme } from './lib/stores/navigation';
  import { authStatus, authLoading, authError, signIn, signOut, checkAuth, discoveryLoading, discoveryError } from './lib/stores/auth';

  const pageTitles: Record<string, string> = {
    home: 'Home', agents: 'Agents', models: 'Models', chat: 'Chat',
    runs: 'Runs', knowledge: 'Knowledge', tools: 'Tools',
    connections: 'Connections', costs: 'Costs & Usage', trust: 'Trust Center',
  };

  let showUserMenu = $state(false);

  onMount(() => {
    checkAuth();
  });

  function userInitial(name: string | null): string {
    return name ? name.charAt(0).toUpperCase() : '?';
  }
</script>

<div class="shell">
  <Sidebar />

  <div class="main">
    <div class="topbar">
      <div class="topbar-title">{pageTitles[$currentPage] || $currentPage}</div>
      <div style="display:flex;align-items:center;gap:6px">
        <div class="search-box" style="width:220px">
          <span style="font-size:.82rem;opacity:.5">🔍</span>
          <input placeholder="Search everything…">
        </div>
        <button class="btn-icon" title="Notifications">🔔</button>
        <button class="btn-icon theme-toggle" title="Toggle theme" onclick={toggleTheme}>
          {#if $theme === 'dark'}☀️{:else}🌙{/if}
        </button>

        {#if $authLoading || $discoveryLoading}
          <span class="auth-status-connecting">⟳ Connecting…</span>
        {:else if $authStatus.signed_in}
          <div class="auth-user-wrapper">
            <button class="auth-user-btn" onclick={() => showUserMenu = !showUserMenu} title={$authStatus.user_name ?? 'User'}>
              <span class="auth-avatar">{userInitial($authStatus.user_name)}</span>
              <span class="auth-user-name">{$authStatus.user_name}</span>
            </button>
            {#if showUserMenu}
              <div class="auth-dropdown">
                <div class="auth-dropdown-header">
                  <span class="auth-avatar" style="width:28px;height:28px;font-size:.72rem">{userInitial($authStatus.user_name)}</span>
                  <div>
                    <div style="font-weight:600;font-size:.82rem">{$authStatus.user_name}</div>
                    <div style="font-size:.68rem;color:var(--text-3)">{$authStatus.auth_mode}</div>
                  </div>
                </div>
                <button class="auth-dropdown-item" onclick={() => { showUserMenu = false; signOut(); }}>Sign Out</button>
              </div>
            {/if}
          </div>
        {:else}
          <button class="btn btn-primary" onclick={signIn} style="font-size:.78rem;padding:5px 12px">Sign In</button>
        {/if}

        <button class="btn btn-primary" onclick={() => navigateTo('chat')} style="font-size:.78rem;padding:5px 12px">+ New Chat</button>
      </div>
    </div>

    {#if $currentPage === 'chat'}
      <ChatPage />
    {:else}
      <div class="content">
        {#if $authError}
          <div class="error-banner" style="margin:12px 20px 0;padding:10px 16px;background:rgba(239,68,68,.1);border:1px solid rgba(239,68,68,.3);border-radius:8px;font-size:.82rem;color:#ef4444;display:flex;align-items:center;gap:8px">
            <span>⚠️</span> Auth error: {$authError}
          </div>
        {/if}
        {#if $discoveryError}
          <div class="error-banner" style="margin:12px 20px 0;padding:10px 16px;background:rgba(245,158,11,.1);border:1px solid rgba(245,158,11,.3);border-radius:8px;font-size:.82rem;color:#f59e0b;display:flex;align-items:center;gap:8px">
            <span>⚠️</span> Discovery failed: {$discoveryError} — some data may not be available.
          </div>
        {/if}
        {#if $currentPage === 'home'}
          <HomePage />
        {:else if $currentPage === 'agents'}
          <AgentsPage />
        {:else if $currentPage === 'models'}
          <ModelsPage />
        {:else if $currentPage === 'runs'}
          <RunsPage />
        {:else if $currentPage === 'knowledge'}
          <KnowledgePage />
        {:else if $currentPage === 'tools'}
          <ToolsPage />
        {:else if $currentPage === 'connections'}
          <ConnectionsPage />
        {:else if $currentPage === 'costs'}
          <CostsPage />
        {:else if $currentPage === 'trust'}
          <TrustPage />
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .auth-status-connecting {
    font-size: .75rem;
    color: var(--text-3);
    animation: auth-spin 1.2s linear infinite;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  @keyframes auth-spin {
    from { opacity: .5; }
    50% { opacity: 1; }
    to { opacity: .5; }
  }
  .auth-user-wrapper {
    position: relative;
  }
  .auth-user-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition);
    background: transparent;
    border: 1px solid var(--border);
  }
  .auth-user-btn:hover {
    background: var(--bg-hover);
  }
  .auth-avatar {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--gradient);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: .65rem;
    font-weight: 700;
    color: #fff;
    flex-shrink: 0;
  }
  .auth-user-name {
    font-size: .78rem;
    font-weight: 500;
    color: var(--text-1);
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .auth-dropdown {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    min-width: 200px;
    background: var(--bg-1);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 8px 24px rgba(0,0,0,.15);
    z-index: 100;
    overflow: hidden;
  }
  .auth-dropdown-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
  }
  .auth-dropdown-item {
    width: 100%;
    text-align: left;
    padding: 10px 14px;
    font-size: .8rem;
    color: var(--text-2);
    cursor: pointer;
    transition: background var(--transition);
    background: transparent;
    border: none;
  }
  .auth-dropdown-item:hover {
    background: var(--bg-hover);
    color: var(--text-1);
  }
</style>
