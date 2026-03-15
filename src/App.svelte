<script lang="ts">
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
  import { currentPage, navigateTo } from './lib/stores/navigation';

  const pageTitles: Record<string, string> = {
    home: 'Home', agents: 'Agents', models: 'Models', chat: 'Chat',
    runs: 'Runs', knowledge: 'Knowledge', tools: 'Tools',
    connections: 'Connections', costs: 'Costs & Usage', trust: 'Trust Center',
  };
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
        <button class="btn btn-primary" onclick={() => navigateTo('chat')} style="font-size:.78rem;padding:5px 12px">+ New Chat</button>
      </div>
    </div>

    {#if $currentPage === 'chat'}
      <ChatPage />
    {:else}
      <div class="content">
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
