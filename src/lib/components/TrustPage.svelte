<div class="page-header" style="text-align:center;padding-top:12px">
  <div style="font-size:3rem;margin-bottom:8px">🛡️</div>
  <div class="page-title">Trust Center</div>
  <div class="page-subtitle">Privacy, security, and local data controls</div>
</div>

<div class="stats-row">
  <div class="stat" style="text-align:center"><div class="stat-val">0</div><div class="stat-label">Active Runs</div></div>
  <div class="stat" style="text-align:center"><div class="stat-val">OAuth</div><div class="stat-label">Auth Mode</div></div>
  <div class="stat" style="text-align:center"><div class="stat-val">~2 KB</div><div class="stat-label">Local Data</div></div>
  <div class="stat" style="text-align:center"><div class="stat-val">0.1</div><div class="stat-label">Version</div></div>
</div>

<div class="section">
  <div class="section-title" style="margin-bottom:14px">Data Handling</div>
  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:14px">
    <div class="card"><div style="font-size:1.3rem;margin-bottom:8px">🔐</div><div style="font-weight:600;font-size:.88rem;margin-bottom:4px">Tokens & Sessions</div><div style="font-size:.78rem;color:var(--text-2)">Stored in OS keychain (macOS Keychain / Windows Credential Manager).</div></div>
    <div class="card"><div style="font-size:1.3rem;margin-bottom:8px">💾</div><div style="font-weight:600;font-size:.88rem;margin-bottom:4px">Workspace Data</div><div style="font-size:.78rem;color:var(--text-2)">Chats, favorites, settings in local app storage.</div></div>
    <div class="card"><div style="font-size:1.3rem;margin-bottom:8px">🌐</div><div style="font-weight:600;font-size:.88rem;margin-bottom:4px">Outbound Services</div><div style="font-size:.78rem;color:var(--text-2)">Azure AI, ARM, Graph, Copilot Studio, local Ollama.</div></div>
    <div class="card"><div style="font-size:1.3rem;margin-bottom:8px">🚫</div><div style="font-weight:600;font-size:.88rem;margin-bottom:4px">No Tracking</div><div style="font-size:.78rem;color:var(--text-2)">No analytics or ad SDKs. Tauri CSP enforced.</div></div>
  </div>
</div>

<div class="section">
  <div class="section-title" style="margin-bottom:14px">Local Controls</div>
  <div class="card" style="padding:0">
    <div style="display:flex;align-items:center;justify-content:space-between;padding:14px 18px;border-bottom:1px solid var(--border)">
      <div><div style="font-weight:500;font-size:.88rem">Clear Saved Chats</div><div style="font-size:.72rem;color:var(--text-3)">Remove all locally saved chat history</div></div>
      <button class="btn btn-outline" style="color:var(--error);border-color:rgba(239,68,68,.3)">Clear</button>
    </div>
    <div style="display:flex;align-items:center;justify-content:space-between;padding:14px 18px;border-bottom:1px solid var(--border)">
      <div><div style="font-weight:500;font-size:.88rem">Clear All Workspace Data</div><div style="font-size:.72rem;color:var(--text-3)">Chats, favorites, settings, onboarding state</div></div>
      <button class="btn btn-outline" style="color:var(--error);border-color:rgba(239,68,68,.3)">Clear All</button>
    </div>
    <div style="display:flex;align-items:center;justify-content:space-between;padding:14px 18px">
      <div><div style="font-weight:500;font-size:.88rem">Sign Out</div><div style="font-size:.72rem;color:var(--text-3)">Remove stored token from OS keychain</div></div>
      <button class="btn btn-outline" style="color:var(--error);border-color:rgba(239,68,68,.3)">Sign Out</button>
    </div>
  </div>
</div>
