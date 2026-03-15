export interface Agent {
  id: string;
  name: string;
  description: string;
  source: 'foundry' | 'studio' | 'm365' | 'local';
  model?: string;
  runtime?: string;
  status: 'active' | 'idle' | 'published' | 'draft' | 'running' | 'offline';
  threads?: number;
  tools?: string[];
  knowledge?: string[];
}

export interface Model {
  id: string;
  name: string;
  version?: string;
  sku?: string;
  region?: string;
  source: 'cloud' | 'local';
  runtime?: string;
  status: 'online' | 'provisioning' | 'failed' | 'offline';
  capabilities?: string[];
}

export interface KnowledgeSource {
  id: string;
  name: string;
  type: 'vector_store' | 'file' | 'sharepoint' | 'onedrive' | 'local_dir';
  size?: string;
  fileCount?: number;
  purpose?: string;
  status: 'ready' | 'syncing' | 'connected' | 'error';
  usedBy?: string[];
  updatedAt?: string;
}

export interface Tool {
  id: string;
  name: string;
  description: string;
  type: 'builtin' | 'azure' | 'openapi' | 'm365';
  usedBy?: string[];
}

export interface Connection {
  id: string;
  name: string;
  description: string;
  category: 'azure' | 'm365' | 'local';
  status: 'connected' | 'configured' | 'error';
}

export interface Run {
  id: string;
  name: string;
  description: string;
  type: 'agent_run' | 'batch' | 'fine_tuning';
  status: 'in_progress' | 'queued' | 'succeeded' | 'completed' | 'failed';
  startedAt?: string;
  progress?: number;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
}

export interface ChatSession {
  id: string;
  title: string;
  targetName: string;
  targetSource: Agent['source'] | 'model';
  targetIcon: string;
  lastMessage?: string;
  updatedAt: string;
}

export interface FeedItem {
  id: string;
  text: string;
  time: string;
  source: string;
  color: 'green' | 'blue' | 'yellow' | 'red';
}

export type Page = 'home' | 'agents' | 'models' | 'chat' | 'runs' | 'knowledge' | 'tools' | 'connections' | 'costs' | 'trust';
