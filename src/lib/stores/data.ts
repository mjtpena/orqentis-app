import type { Agent, Model, KnowledgeSource, Tool, Connection, Run, ChatSession, FeedItem } from '../types';

export const agents: Agent[] = [
  { id: 'a1', name: 'support-agent', description: 'Customer support triage and resolution with file search and code interpreter tools', source: 'foundry', model: 'gpt-4o', status: 'active', threads: 3, tools: ['file_search', 'code_interpreter', 'jira'], knowledge: ['product-knowledge', 'support-docs'] },
  { id: 'a2', name: 'data-analyst', description: 'SQL generation, chart rendering, data interpretation', source: 'foundry', model: 'gpt-4o-mini', status: 'idle', tools: ['code_interpreter'] },
  { id: 'a3', name: 'rag-pipeline', description: 'Retrieval-augmented generation over product knowledge base', source: 'foundry', model: 'gpt-4o', status: 'idle', threads: 1, tools: ['file_search', 'azure_ai_search'], knowledge: ['product-knowledge'] },
  { id: 'a4', name: 'HR Assistant', description: 'Employee onboarding, leave requests, and policy questions via Copilot Studio', source: 'studio', status: 'published', knowledge: ['policy-handbook'] },
  { id: 'a5', name: 'IT Helpdesk Bot', description: 'Password resets, VPN setup, software requests', source: 'studio', status: 'draft' },
  { id: 'a6', name: 'Meeting Summarizer', description: 'Summarizes Teams meetings, extracts action items, and drafts follow-up emails', source: 'm365', status: 'active' },
  { id: 'a7', name: 'code-reviewer', description: 'Runs locally with Ollama. Reviews PRs, suggests fixes, no data leaves your machine', source: 'local', model: 'llama3.3', runtime: 'Ollama', status: 'running' },
];

export const models: Model[] = [
  { id: 'm1', name: 'gpt-4o', version: 'v2024-08-06', sku: 'Standard', region: 'East US', source: 'cloud', status: 'online', capabilities: ['chat', 'vision'] },
  { id: 'm2', name: 'gpt-4o-mini', version: 'v2024-07-18', sku: 'Standard', region: 'East US', source: 'cloud', status: 'online', capabilities: ['chat'] },
  { id: 'm3', name: 'gpt-4.1', version: 'v2025-04-14', sku: 'GlobalStandard', source: 'cloud', status: 'provisioning' },
  { id: 'm4', name: 'o3-mini', version: 'v2025-01-31', sku: 'DataZone', source: 'cloud', status: 'failed' },
  { id: 'm5', name: 'text-embedding-3-large', version: 'v1', sku: 'Standard', region: 'East US', source: 'cloud', status: 'online', capabilities: ['embeddings'] },
  { id: 'm6', name: 'Phi-4-multimodal', sku: 'Serverless', source: 'cloud', status: 'online', capabilities: ['chat', 'vision'] },
  { id: 'm7', name: 'dall-e-3', version: 'v3.0', sku: 'Standard', source: 'cloud', status: 'online', capabilities: ['image'] },
  { id: 'm8', name: 'whisper', version: 'v1', sku: 'Standard', source: 'cloud', status: 'online', capabilities: ['audio'] },
  { id: 'm9', name: 'llama3.3:70b', source: 'local', runtime: 'Ollama · localhost:11434', status: 'online', capabilities: ['chat'] },
  { id: 'm10', name: 'nomic-embed-text', source: 'local', runtime: 'Ollama · localhost:11434', status: 'online', capabilities: ['embeddings'] },
];

export const knowledge: KnowledgeSource[] = [
  { id: 'k1', name: 'product-knowledge', type: 'vector_store', fileCount: 12, size: '48 MB', status: 'ready', usedBy: ['support-agent', 'rag-pipeline'] },
  { id: 'k2', name: 'support-docs', type: 'vector_store', fileCount: 6, size: '22 MB', status: 'ready', usedBy: ['support-agent'] },
  { id: 'k3', name: 'policy-handbook', type: 'vector_store', fileCount: 3, size: '8 MB', status: 'ready', usedBy: ['HR Assistant'] },
  { id: 'k4', name: 'training-data-v3.jsonl', type: 'file', size: '2.4 MB', purpose: 'fine_tuning', status: 'ready', updatedAt: '2d ago' },
  { id: 'k5', name: 'batch-input-march.jsonl', type: 'file', size: '890 KB', purpose: 'batch', status: 'ready', updatedAt: '5d ago' },
  { id: 'k6', name: 'product-catalog.pdf', type: 'file', size: '14.2 MB', purpose: 'assistants', status: 'ready', updatedAt: '1w ago' },
  { id: 'k7', name: 'SharePoint: Contoso Policies', type: 'sharepoint', fileCount: 42, status: 'ready', usedBy: ['HR Assistant'], updatedAt: 'Auto-synced daily' },
  { id: 'k8', name: 'OneDrive: AI Project Files', type: 'onedrive', fileCount: 18, status: 'connected', usedBy: ['Meeting Summarizer'], updatedAt: 'On-demand' },
];

export const tools: Tool[] = [
  { id: 't1', name: 'File Search', description: 'Searches across vector stores and attached files to provide grounded answers', type: 'builtin', usedBy: ['support-agent', 'rag-pipeline'] },
  { id: 't2', name: 'Code Interpreter', description: 'Executes Python code in a sandboxed environment for data analysis and visualization', type: 'builtin', usedBy: ['data-analyst'] },
  { id: 't3', name: 'Bing Search', description: 'Real-time web search grounding for up-to-date information retrieval', type: 'azure', usedBy: ['support-agent'] },
  { id: 't4', name: 'Azure AI Search', description: 'Enterprise search with semantic ranking, filters, and hybrid retrieval', type: 'azure', usedBy: ['rag-pipeline'] },
  { id: 't5', name: 'Custom API: Jira', description: 'Create tickets, query issues, update status via Jira REST API', type: 'openapi', usedBy: ['support-agent'] },
  { id: 't6', name: 'Microsoft Graph', description: 'Calendar, mail, files, people, and Teams data for M365-aware agents', type: 'm365', usedBy: ['Meeting Summarizer', 'HR Assistant'] },
];

export const connections: Connection[] = [
  { id: 'c1', name: 'Azure OpenAI — ai-services-eastus', description: 'Default AI Services · East US', category: 'azure', status: 'connected' },
  { id: 'c2', name: 'Azure AI Search — search-eastus', description: 'Semantic index · Standard S1', category: 'azure', status: 'connected' },
  { id: 'c3', name: 'Azure Blob Storage — stfoundryprod', description: 'Training data & outputs', category: 'azure', status: 'connected' },
  { id: 'c4', name: 'Azure Key Vault — kv-foundry-prod', description: 'Secrets & certificates', category: 'azure', status: 'connected' },
  { id: 'c5', name: 'Microsoft Graph', description: 'Users, Groups, Mail, Calendar, Teams, Files', category: 'm365', status: 'connected' },
  { id: 'c6', name: 'Copilot Studio — contoso.crm.dynamics.com', description: 'Power Platform environment', category: 'm365', status: 'connected' },
  { id: 'c7', name: 'Ollama — localhost:11434', description: '2 models loaded · 40 GB VRAM', category: 'local', status: 'connected' },
  { id: 'c8', name: 'Local File System — ~/Documents/ai-data', description: 'Read-only access for knowledge ingestion', category: 'local', status: 'configured' },
];

export const runs: Run[] = [
  { id: 'r1', name: 'support-agent · Thread #t-482', description: 'Foundry Agent Run · Started 12 min ago', type: 'agent_run', status: 'in_progress' },
  { id: 'r2', name: 'batch-classify · 5,000 items', description: 'Foundry Batch · /chat/completions · 62% complete', type: 'batch', status: 'in_progress', progress: 62 },
  { id: 'r3', name: 'ft-gpt4o-support-v2', description: 'Fine-Tuning · 3 epochs · Completed 2d ago', type: 'fine_tuning', status: 'succeeded' },
  { id: 'r4', name: 'batch-march-embeddings · 1,200 items', description: 'Foundry Batch · /embeddings · Completed 5d ago', type: 'batch', status: 'completed' },
  { id: 'r5', name: 'support-agent · Thread #t-471', description: 'Foundry Agent Run · Completed 1d ago', type: 'agent_run', status: 'succeeded' },
  { id: 'r6', name: 'ft-embed-custom', description: 'Fine-Tuning · Failed during validation · 3d ago', type: 'fine_tuning', status: 'failed' },
];

export const chatSessions: ChatSession[] = [
  { id: 's1', title: 'Summarize Q4 report', targetName: 'gpt-4o', targetSource: 'model', targetIcon: '🧠', updatedAt: '2h ago' },
  { id: 's2', title: 'Ticket #4821 triage', targetName: 'support-agent', targetSource: 'foundry', targetIcon: '🤖', updatedAt: '12m ago' },
  { id: 's3', title: 'Review PR #287', targetName: 'code-reviewer', targetSource: 'local', targetIcon: '🏠', updatedAt: '1d ago' },
  { id: 's4', title: 'Leave policy question', targetName: 'HR Assistant', targetSource: 'studio', targetIcon: '✨', updatedAt: '2d ago' },
];

export const activityFeed: FeedItem[] = [
  { id: 'f1', text: '<strong>support-agent</strong> completed run — resolved ticket #4821', time: '12 min ago', source: 'Foundry', color: 'green' },
  { id: 'f2', text: 'Chat with <strong>gpt-4o</strong> — "Summarize Q4 report"', time: '2 hours ago', source: 'Foundry Model', color: 'blue' },
  { id: 'f3', text: '<strong>HR Assistant</strong> handled 14 leave requests today', time: '3 hours ago', source: 'Copilot Studio', color: 'blue' },
  { id: 'f4', text: '<strong>batch-classify</strong> job started — 5,000 items queued', time: '4 hours ago', source: 'Foundry Batch', color: 'yellow' },
  { id: 'f5', text: '<strong>product-knowledge</strong> vector store updated — 3 new files indexed', time: 'Yesterday', source: 'Knowledge', color: 'green' },
  { id: 'f6', text: '<strong>code-reviewer</strong> local agent started on Ollama', time: 'Yesterday', source: 'Local', color: 'green' },
];
