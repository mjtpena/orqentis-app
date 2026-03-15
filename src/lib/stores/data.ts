// This module previously contained hardcoded mock data.
// All pages now load live data from the Tauri backend when signed in,
// and show appropriate empty states when not connected.

import type { ChatSession, FeedItem } from '../types';

// Chat sessions are client-side only (not persisted to backend yet)
import { writable } from 'svelte/store';
export const chatSessions = writable<ChatSession[]>([]);

// Activity feed is derived from real events — currently empty until
// event tracking is implemented.
export const activityFeed = writable<FeedItem[]>([]);
