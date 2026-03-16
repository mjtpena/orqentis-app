import { writable, derived } from "svelte/store";
import type {
  AuthStatus,
  DiscoveryResult,
  HubDetail,
  StudioBot,
  M365Agent,
  LocalAgent,
} from "../services/api";
import * as api from "../services/api";

export const authStatus = writable<AuthStatus>({
  signed_in: false,
  user_name: null,
  tenant_id: null,
  auth_mode: "none",
});

export const authLoading = writable(false);
export const authError = writable<string | null>(null);

export const discoveryResult = writable<DiscoveryResult | null>(null);
export const discoveryLoading = writable(false);
export const discoveryError = writable<string | null>(null);

// Which hub is currently selected (index into hubs array); defaults to first with endpoint
export const selectedHubIndex = writable(0);

// Derived: all hubs
export const hubs = derived(discoveryResult, ($dr) => {
  return $dr?.hubs ?? [];
});

// Derived: currently selected hub detail
export const activeHub = derived([hubs, selectedHubIndex], ([$hubs, $idx]) => {
  if ($hubs.length === 0) return null;
  // Try the selected index, otherwise find first with endpoint
  if ($hubs[$idx]?.endpoint) return $hubs[$idx];
  return $hubs.find((h) => h.endpoint) ?? $hubs[0] ?? null;
});

// Derived: the currently active endpoint
export const activeEndpoint = derived(activeHub, ($hub) => {
  return $hub?.endpoint ?? null;
});

// Derived: subscription count
export const subscriptionCount = derived(
  discoveryResult,
  ($dr) => $dr?.subscriptions.length ?? 0,
);

// Derived: total workspace count
export const workspaceCount = derived(
  discoveryResult,
  ($dr) => $dr?.workspaces.length ?? 0,
);

// Derived: ARM deployments from the active hub
export const armDeployments = derived(activeHub, ($hub) => {
  return $hub?.deployments ?? [];
});

// Derived: projects from the active hub
export const activeProjects = derived(activeHub, ($hub) => {
  return $hub?.projects ?? [];
});

// Multi-source agent stores
export const studioAgents = writable<StudioBot[]>([]);
export const studioLoading = writable(false);
export const studioError = writable<string | null>(null);

export const m365Agents = writable<M365Agent[]>([]);
export const m365Loading = writable(false);
export const m365Error = writable<string | null>(null);

export const localAgents = writable<LocalAgent[]>([]);
export const localLoading = writable(false);
export const localError = writable<string | null>(null);

export async function checkAuth() {
  authLoading.set(true);
  authError.set(null);
  try {
    const status = await api.getAuthStatus();
    console.log("[auth] checkAuth status:", status);
    authStatus.set(status);
    if (status.signed_in) {
      await discover();
    }
  } catch (e: any) {
    console.error("[auth] checkAuth failed:", e);
    authError.set(e?.toString() ?? "Unknown error");
  } finally {
    authLoading.set(false);
  }
}

export async function signIn() {
  authLoading.set(true);
  authError.set(null);
  try {
    const status = await api.signIn();
    console.log("[auth] signIn status:", status);
    authStatus.set(status);
    if (status.signed_in) {
      await discover();
    }
  } catch (e: any) {
    console.error("[auth] signIn failed:", e);
    authError.set(e?.toString() ?? "Sign-in failed");
  } finally {
    authLoading.set(false);
  }
}

export async function signOut() {
  try {
    await api.signOut();
    authStatus.set({
      signed_in: false,
      user_name: null,
      tenant_id: null,
      auth_mode: "none",
    });
    discoveryResult.set(null);
    studioAgents.set([]);
    m365Agents.set([]);
    localAgents.set([]);
  } catch (e: any) {
    authError.set(e?.toString() ?? "Sign-out failed");
  }
}

export async function discover() {
  discoveryLoading.set(true);
  discoveryError.set(null);
  try {
    const result = await api.discoverResources();
    console.log("[auth] discover result:", result);
    discoveryResult.set(result);
  } catch (e: any) {
    console.error("[auth] discover failed:", e);
    discoveryError.set(e?.toString() ?? "Discovery failed");
  } finally {
    discoveryLoading.set(false);
  }

  // Load additional sources in parallel (non-blocking)
  loadStudioAgents();
  loadM365Agents();
  loadLocalAgents();
}

export async function loadStudioAgents() {
  studioLoading.set(true);
  studioError.set(null);
  try {
    const bots = await api.listStudioAgents();
    console.log("[auth] studio agents:", bots.length);
    studioAgents.set(bots);
  } catch (e: any) {
    console.warn(
      "[auth] studio agents failed (expected if no Graph consent):",
      e,
    );
    studioError.set(e?.toString() ?? "Studio discovery failed");
  } finally {
    studioLoading.set(false);
  }
}

export async function loadM365Agents() {
  m365Loading.set(true);
  m365Error.set(null);
  try {
    const agents = await api.listM365Agents();
    console.log("[auth] m365 agents:", agents.length);
    m365Agents.set(agents);
  } catch (e: any) {
    console.warn(
      "[auth] m365 agents failed (expected if no Graph consent):",
      e,
    );
    m365Error.set(e?.toString() ?? "M365 discovery failed");
  } finally {
    m365Loading.set(false);
  }
}

export async function loadLocalAgents() {
  localLoading.set(true);
  localError.set(null);
  try {
    const agents = await api.listLocalAgents();
    console.log("[auth] local agents:", agents.length);
    localAgents.set(agents);
  } catch (e: any) {
    console.warn("[auth] local agents failed:", e);
    localError.set(e?.toString() ?? "Local discovery failed");
  } finally {
    localLoading.set(false);
  }
}
