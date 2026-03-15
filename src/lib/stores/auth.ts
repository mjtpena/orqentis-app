import { writable, derived } from 'svelte/store';
import type { AuthStatus, DiscoveryResult, HubDetail } from '../services/api';
import * as api from '../services/api';

export const authStatus = writable<AuthStatus>({
  signed_in: false,
  user_name: null,
  tenant_id: null,
  auth_mode: 'none',
});

export const authLoading = writable(false);
export const authError = writable<string | null>(null);

export const discoveryResult = writable<DiscoveryResult | null>(null);
export const discoveryLoading = writable(false);
export const discoveryError = writable<string | null>(null);

// Derived: the currently active endpoint (first hub with an endpoint)
export const activeEndpoint = derived(discoveryResult, ($dr) => {
  if (!$dr) return null;
  for (const hub of $dr.hubs) {
    if (hub.endpoint) return hub.endpoint;
  }
  return null;
});

// Derived: all hubs
export const hubs = derived(discoveryResult, ($dr) => {
  return $dr?.hubs ?? [];
});

export async function checkAuth() {
  authLoading.set(true);
  authError.set(null);
  try {
    const status = await api.getAuthStatus();
    authStatus.set(status);
    if (status.signed_in) {
      await discover();
    }
  } catch (e: any) {
    authError.set(e?.toString() ?? 'Unknown error');
  } finally {
    authLoading.set(false);
  }
}

export async function signIn() {
  authLoading.set(true);
  authError.set(null);
  try {
    const status = await api.signIn();
    authStatus.set(status);
    if (status.signed_in) {
      await discover();
    }
  } catch (e: any) {
    authError.set(e?.toString() ?? 'Sign-in failed');
  } finally {
    authLoading.set(false);
  }
}

export async function signOut() {
  try {
    await api.signOut();
    authStatus.set({ signed_in: false, user_name: null, tenant_id: null, auth_mode: 'none' });
    discoveryResult.set(null);
  } catch (e: any) {
    authError.set(e?.toString() ?? 'Sign-out failed');
  }
}

export async function discover() {
  discoveryLoading.set(true);
  discoveryError.set(null);
  try {
    const result = await api.discoverResources();
    discoveryResult.set(result);
  } catch (e: any) {
    discoveryError.set(e?.toString() ?? 'Discovery failed');
  } finally {
    discoveryLoading.set(false);
  }
}
