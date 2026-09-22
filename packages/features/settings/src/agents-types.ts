export type AgentToolCallRow = {
  id: string;
  projectId: string;
  agentRunId: string;
  toolName: string;
  argumentsJson: string;
  resultSummary: string;
  ok: boolean;
  source: string;
  createdAt: string;
};

export type AgentRunRow = {
  id: string;
  projectId: string;
  task: string;
  agentLabel: string;
  status: string;
  createdAt: string;
  completedAt: string;
};

export type AgentRunStepRow = {
  id: string;
  agentRunId: string;
  kind: string;
  summary: string;
  payloadJson: string;
  createdAt: string;
};

export type AgentRunDetail = {
  run: AgentRunRow;
  steps: AgentRunStepRow[];
};

export type CompilePreviewEvidence = {
  id: string;
  kind: string;
  summary: string;
  path: string;
  source: string;
};

export type CompilePreviewItem = {
  kind: string;
  id: string;
  title: string;
  excerpt: string;
  truthClass: string;
  source: string;
  path: string;
  reasons: string[];
  evidence: CompilePreviewEvidence[];
};

export type CompilePreviewResult = {
  tokensUsed: number;
  tokenBudget: number;
  truncated: boolean;
  items: CompilePreviewItem[];
};

export type McpStatusView = {
  running: boolean;
  pid: number | null;
  dbPath: string;
  command: string;
  cursorConfig: string;
};
