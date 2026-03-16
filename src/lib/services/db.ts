import Database from "@tauri-apps/plugin-sql";

let db: Database | null = null;

export async function getDb(): Promise<Database> {
  if (db) return db;
  db = await Database.load("sqlite:orqentis.db");
  await migrate(db);
  return db;
}

async function migrate(db: Database): Promise<void> {
  await db.execute(`
    CREATE TABLE IF NOT EXISTS chat_sessions (
      id TEXT PRIMARY KEY,
      title TEXT NOT NULL,
      target_name TEXT NOT NULL,
      target_source TEXT NOT NULL,
      target_icon TEXT NOT NULL DEFAULT '🧠',
      last_message TEXT,
      created_at TEXT NOT NULL DEFAULT (datetime('now')),
      updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    )
  `);
  await db.execute(`
    CREATE TABLE IF NOT EXISTS chat_messages (
      id TEXT PRIMARY KEY,
      session_id TEXT NOT NULL,
      role TEXT NOT NULL,
      content TEXT NOT NULL,
      timestamp TEXT NOT NULL DEFAULT (datetime('now')),
      FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
    )
  `);
  await db.execute(`
    CREATE INDEX IF NOT EXISTS idx_messages_session
    ON chat_messages(session_id, timestamp)
  `);
  await db.execute(`
    CREATE TABLE IF NOT EXISTS cached_agents (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT,
      source TEXT NOT NULL,
      model TEXT,
      runtime TEXT,
      status TEXT NOT NULL DEFAULT 'offline',
      data_json TEXT,
      cached_at TEXT NOT NULL DEFAULT (datetime('now'))
    )
  `);
  await db.execute(`
    CREATE TABLE IF NOT EXISTS cached_deployments (
      name TEXT PRIMARY KEY,
      model_name TEXT,
      model_version TEXT,
      sku_name TEXT,
      sku_capacity INTEGER,
      provisioning_state TEXT,
      data_json TEXT,
      cached_at TEXT NOT NULL DEFAULT (datetime('now'))
    )
  `);
  await db.execute(`
    CREATE TABLE IF NOT EXISTS settings (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    )
  `);
}

// ---- Chat Sessions ----
export interface DbChatSession {
  id: string;
  title: string;
  target_name: string;
  target_source: string;
  target_icon: string;
  last_message: string | null;
  created_at: string;
  updated_at: string;
}

export interface DbChatMessage {
  id: string;
  session_id: string;
  role: string;
  content: string;
  timestamp: string;
}

export async function listChatSessions(): Promise<DbChatSession[]> {
  const db = await getDb();
  return db.select<DbChatSession[]>(
    "SELECT * FROM chat_sessions ORDER BY updated_at DESC",
  );
}

export async function createChatSession(session: DbChatSession): Promise<void> {
  const db = await getDb();
  await db.execute(
    `INSERT INTO chat_sessions (id, title, target_name, target_source, target_icon, last_message, created_at, updated_at)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
    [
      session.id,
      session.title,
      session.target_name,
      session.target_source,
      session.target_icon,
      session.last_message,
      session.created_at,
      session.updated_at,
    ],
  );
}

export async function updateChatSession(
  id: string,
  lastMessage: string,
): Promise<void> {
  const db = await getDb();
  await db.execute(
    `UPDATE chat_sessions SET last_message = $1, updated_at = datetime('now') WHERE id = $2`,
    [lastMessage, id],
  );
}

export async function deleteChatSession(id: string): Promise<void> {
  const db = await getDb();
  await db.execute("DELETE FROM chat_messages WHERE session_id = $1", [id]);
  await db.execute("DELETE FROM chat_sessions WHERE id = $1", [id]);
}

// ---- Chat Messages ----
export async function getSessionMessages(
  sessionId: string,
): Promise<DbChatMessage[]> {
  const db = await getDb();
  return db.select<DbChatMessage[]>(
    "SELECT * FROM chat_messages WHERE session_id = $1 ORDER BY timestamp ASC",
    [sessionId],
  );
}

export async function saveMessage(msg: DbChatMessage): Promise<void> {
  const db = await getDb();
  await db.execute(
    `INSERT OR REPLACE INTO chat_messages (id, session_id, role, content, timestamp)
     VALUES ($1, $2, $3, $4, $5)`,
    [msg.id, msg.session_id, msg.role, msg.content, msg.timestamp],
  );
}

export async function updateMessageContent(
  id: string,
  content: string,
): Promise<void> {
  const db = await getDb();
  await db.execute("UPDATE chat_messages SET content = $1 WHERE id = $2", [
    content,
    id,
  ]);
}

// ---- Cached Agents ----
export async function cacheAgents(
  agents: Array<{
    id: string;
    name: string;
    description?: string;
    source: string;
    model?: string;
    runtime?: string;
    status: string;
  }>,
): Promise<void> {
  const db = await getDb();
  for (const a of agents) {
    await db.execute(
      `INSERT OR REPLACE INTO cached_agents (id, name, description, source, model, runtime, status, data_json, cached_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, datetime('now'))`,
      [
        a.id,
        a.name,
        a.description ?? null,
        a.source,
        a.model ?? null,
        a.runtime ?? null,
        a.status,
        JSON.stringify(a),
      ],
    );
  }
}

export async function getCachedAgents(): Promise<
  Array<{
    id: string;
    name: string;
    description: string | null;
    source: string;
    model: string | null;
    runtime: string | null;
    status: string;
  }>
> {
  const db = await getDb();
  return db.select(
    "SELECT id, name, description, source, model, runtime, status FROM cached_agents ORDER BY name",
  );
}

// ---- Cached Deployments ----
export async function cacheDeployments(
  deployments: Array<{
    name: string;
    model_name?: string;
    model_version?: string;
    sku_name?: string;
    sku_capacity?: number;
    provisioning_state?: string;
  }>,
): Promise<void> {
  const db = await getDb();
  for (const d of deployments) {
    await db.execute(
      `INSERT OR REPLACE INTO cached_deployments (name, model_name, model_version, sku_name, sku_capacity, provisioning_state, data_json, cached_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, datetime('now'))`,
      [
        d.name,
        d.model_name ?? null,
        d.model_version ?? null,
        d.sku_name ?? null,
        d.sku_capacity ?? null,
        d.provisioning_state ?? null,
        JSON.stringify(d),
      ],
    );
  }
}

export async function getCachedDeployments(): Promise<
  Array<{
    name: string;
    model_name: string | null;
    model_version: string | null;
    sku_name: string | null;
    sku_capacity: number | null;
    provisioning_state: string | null;
  }>
> {
  const db = await getDb();
  return db.select(
    "SELECT name, model_name, model_version, sku_name, sku_capacity, provisioning_state FROM cached_deployments ORDER BY name",
  );
}

// ---- Settings ----
export async function getSetting(key: string): Promise<string | null> {
  const db = await getDb();
  const rows = await db.select<Array<{ value: string }>>(
    "SELECT value FROM settings WHERE key = $1",
    [key],
  );
  return rows.length > 0 ? rows[0].value : null;
}

export async function setSetting(key: string, value: string): Promise<void> {
  const db = await getDb();
  await db.execute(
    "INSERT OR REPLACE INTO settings (key, value) VALUES ($1, $2)",
    [key, value],
  );
}
