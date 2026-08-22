import type { ToolErrorCategory, ToolResultStatus } from "./agent-tool-result-contract";

export type { ToolErrorCategory, ToolResultStatus } from "./agent-tool-result-contract";

export interface ToolErrorInfo {
  code: string;
  category: ToolErrorCategory;
  /** Une relance est sûre sans vérification préalable de l'état externe. */
  retryable: boolean;
  hint?: string;
}

export interface PersistedToolResultMeta {
  status: ToolResultStatus;
  error?: ToolErrorInfo;
  warnings?: string[];
  truncated?: boolean;
}
