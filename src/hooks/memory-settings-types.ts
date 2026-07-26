export type MemoryMode = "disabled" | "manual" | "automatic";

export interface MemoryTopic {
  id: string;
  title: string;
  summary: string;
  memoryType: "preference" | "feedback" | "project" | "reference";
  status: "confirmed" | "inferred" | "stale" | "archived";
  tags: string[];
  createdAt: string;
  updatedAt: string;
  source: string;
  sessionId: string;
  path: string;
}

export interface MemoryScopeOverview {
  id: string;
  label: string;
  topicCount: number;
  totalBytes: number;
  lastUpdated?: string;
  topics: MemoryTopic[];
  topicsLoaded: boolean;
}

export interface MemoryOverview {
  settings: { mode: MemoryMode; contextBudgetTokens: number };
  global: MemoryScopeOverview;
  activeProject?: MemoryScopeOverview;
  otherProjects: MemoryScopeOverview[];
  legacyDetected: boolean;
}
