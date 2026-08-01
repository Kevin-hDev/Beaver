export type ToolResultStatus =
  | "success"
  | "running"
  | "partial"
  | "error"
  | "cancelled"
  | "stopped";

export type ToolErrorCategory =
  | "validation"
  | "permission"
  | "not_found"
  | "conflict"
  | "timeout"
  | "cancelled"
  | "unavailable"
  | "external"
  | "execution"
  | "internal";

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
