import { invoke } from '@tauri-apps/api/core';

// Types matching Rust return types
export interface AuthStatus {
  signed_in: boolean;
  user_name: string | null;
  tenant_id: string | null;
  auth_mode: 'oauth' | 'az_cli' | 'none';
}

export interface AuthConfig {
  tenant_id: string;
  client_id: string;
}

export interface Subscription {
  subscription_id: string;
  display_name: string;
  state: string;
}

export interface Workspace {
  id: string;
  name: string;
  location: string;
  kind: string;
  properties: {
    hub_resource_id?: string;
  };
}

export interface HubDetail {
  hub: Workspace;
  projects: Workspace[];
  endpoint: string | null;
  deployments: ArmDeployment[];
  connections: ArmConnection[];
}

export interface DiscoveryResult {
  subscriptions: Subscription[];
  hubs: HubDetail[];
}

export interface ArmDeployment {
  name: string;
  properties: {
    model?: { name: string; version: string; format: string };
    provisioning_state: string;
    sku?: { name: string; capacity: number };
  };
}

export interface ArmConnection {
  id: string;
  name: string;
  properties: {
    category: string;
    target?: string;
  };
}

// Foundry types (matching Rust foundry.rs serde output)
export interface FoundryDeployment {
  id: string;
  model: { name: string; version?: string };
  status: string;
  sku?: { name: string; capacity?: number };
}

export interface FoundryAgent {
  id: string;
  name: string;
  model: string;
  instructions?: string;
  tools: Array<{ type: string }>;
  created_at: number;
}

export interface FoundryFile {
  id: string;
  filename: string;
  bytes: number;
  purpose: string;
  status: string;
  created_at: number;
}

export interface VectorStore {
  id: string;
  name: string;
  status: string;
  file_counts: { total: number };
  usage_bytes: number;
  created_at: number;
}

export interface FineTuningJob {
  id: string;
  model: string;
  status: string;
  training_file: string;
  created_at: number;
  finished_at?: number;
}

export interface BatchJob {
  id: string;
  endpoint: string;
  status: string;
  input_file_id: string;
  completion_window: string;
  created_at: number;
}

export interface FoundryModel {
  id: string;
  object: string;
  created: number;
  owned_by: string;
}

export interface FoundryConnection {
  id: string;
  name: string;
  properties: {
    category: string;
    target?: string;
  };
}

export interface ChatMessagePayload {
  role: string;
  content: string;
}

export interface AgentRunResult {
  thread_id: string;
  run_id: string;
  status: string;
  messages: ChatMessagePayload[];
}

// ---- Auth ----
export async function signIn(): Promise<AuthStatus> {
  return invoke('sign_in');
}

export async function signOut(): Promise<void> {
  return invoke('sign_out');
}

export async function getAuthStatus(): Promise<AuthStatus> {
  return invoke('get_auth_status');
}

export async function getAuthConfig(): Promise<AuthConfig> {
  return invoke('get_auth_config');
}

// ---- Discovery ----
export async function discoverResources(): Promise<DiscoveryResult> {
  return invoke('discover_resources');
}

export async function listSubscriptions(): Promise<Subscription[]> {
  return invoke('list_subscriptions');
}

export async function listArmDeployments(aiServicesResourceId: string): Promise<ArmDeployment[]> {
  return invoke('list_deployments', { aiServicesResourceId });
}

// ---- Foundry ----
export async function listFoundryDeployments(endpoint: string): Promise<FoundryDeployment[]> {
  return invoke('list_foundry_deployments', { endpoint });
}

export async function listAgents(endpoint: string): Promise<FoundryAgent[]> {
  return invoke('list_agents', { endpoint });
}

export async function createAgent(endpoint: string, name: string, model: string, instructions: string): Promise<FoundryAgent> {
  return invoke('create_agent', { endpoint, name, model, instructions });
}

export async function deleteAgent(endpoint: string, agentId: string): Promise<void> {
  return invoke('delete_agent', { endpoint, agentId });
}

export async function listFiles(endpoint: string): Promise<FoundryFile[]> {
  return invoke('list_files', { endpoint });
}

export async function deleteFile(endpoint: string, fileId: string): Promise<void> {
  return invoke('delete_file', { endpoint, fileId });
}

export async function listVectorStores(endpoint: string): Promise<VectorStore[]> {
  return invoke('list_vector_stores', { endpoint });
}

export async function listFineTuningJobs(endpoint: string): Promise<FineTuningJob[]> {
  return invoke('list_fine_tuning_jobs', { endpoint });
}

export async function listBatchJobs(endpoint: string): Promise<BatchJob[]> {
  return invoke('list_batch_jobs', { endpoint });
}

export async function listConnections(endpoint: string): Promise<FoundryConnection[]> {
  return invoke('list_connections', { endpoint });
}

export async function listModels(endpoint: string): Promise<FoundryModel[]> {
  return invoke('list_models', { endpoint });
}

// ---- Chat ----
// Note: send_chat_message streams via events, not return value
export async function sendChatMessage(endpoint: string, deploymentName: string, messages: ChatMessagePayload[]): Promise<void> {
  return invoke('send_chat_message', { endpoint, deploymentName, messages });
}

export async function sendAgentMessage(endpoint: string, agentId: string, threadId: string | null, message: string): Promise<AgentRunResult> {
  return invoke('send_agent_message', { endpoint, agentId, threadId, message });
}
