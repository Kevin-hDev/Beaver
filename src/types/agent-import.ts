export type AgentImportItemKind = "document" | "rule" | "skill";
export type AgentSourceStatus = "detected" | "empty" | "missing" | "unavailable";
export type AgentSelectionMode = "all" | "none" | "custom";

export interface AgentImportItem {
  id: string;
  name: string;
  description: string;
  sourceId: string;
  sourceName: string;
  kind: AgentImportItemKind;
  selected: boolean;
  available: boolean;
  updateAvailable: boolean;
}

export interface AgentSourceSummary {
  id: string;
  displayName: string;
  status: AgentSourceStatus;
  partial: boolean;
  configured: boolean;
  enabled: boolean;
  documents: AgentImportItem[];
  rules: AgentImportItem[];
  skills: AgentImportItem[];
}

export interface AgentSourceSelection {
  sourceId: string;
  enabled: boolean;
  skillMode: AgentSelectionMode;
  selectedSkillIds: string[];
  selectedRuleIds: string[];
  selectedDocumentIds: string[];
}

export interface SaveAgentSourceResult {
  saved: boolean;
  conflicts: string[];
}
