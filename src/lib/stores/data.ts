import type { ChatSession, FeedItem } from "../types";
import { writable, get } from "svelte/store";
import * as db from "../services/db";

// Chat sessions are now persisted to SQLite
export const chatSessions = writable<ChatSession[]>([]);

// Activity feed is derived from real events
export const activityFeed = writable<FeedItem[]>([]);

// Load chat sessions from SQLite on startup
export async function loadChatSessions(): Promise<void> {
  try {
    const rows = await db.listChatSessions();
    const sessions: ChatSession[] = rows.map((r) => ({
      id: r.id,
      title: r.title,
      targetName: r.target_name,
      targetSource: r.target_source as ChatSession["targetSource"],
      targetIcon: r.target_icon,
      lastMessage: r.last_message ?? undefined,
      updatedAt: r.updated_at,
    }));
    chatSessions.set(sessions);
  } catch (e) {
    console.error("[data] Failed to load chat sessions:", e);
  }
}

export async function createSession(session: ChatSession): Promise<void> {
  await db.createChatSession({
    id: session.id,
    title: session.title,
    target_name: session.targetName,
    target_source: session.targetSource,
    target_icon: session.targetIcon,
    last_message: session.lastMessage ?? null,
    created_at: new Date().toISOString(),
    updated_at: session.updatedAt,
  });
  chatSessions.update((s) => [session, ...s]);
}

export async function updateSessionLastMessage(
  id: string,
  lastMessage: string,
): Promise<void> {
  await db.updateChatSession(id, lastMessage);
  chatSessions.update((sessions) =>
    sessions.map((s) =>
      s.id === id
        ? { ...s, lastMessage, updatedAt: new Date().toISOString() }
        : s,
    ),
  );
}

export async function deleteSession(id: string): Promise<void> {
  await db.deleteChatSession(id);
  chatSessions.update((s) => s.filter((sess) => sess.id !== id));
}

// Load messages for a specific session
export async function loadSessionMessages(
  sessionId: string,
): Promise<
  Array<{ id: string; role: string; content: string; timestamp: Date }>
> {
  const rows = await db.getSessionMessages(sessionId);
  return rows.map((r) => ({
    id: r.id,
    role: r.role,
    content: r.content,
    timestamp: new Date(r.timestamp),
  }));
}

export async function persistMessage(
  sessionId: string,
  msg: { id: string; role: string; content: string; timestamp: Date },
): Promise<void> {
  await db.saveMessage({
    id: msg.id,
    session_id: sessionId,
    role: msg.role,
    content: msg.content,
    timestamp: msg.timestamp.toISOString(),
  });
}

export async function persistMessageUpdate(
  id: string,
  content: string,
): Promise<void> {
  await db.updateMessageContent(id, content);
}
