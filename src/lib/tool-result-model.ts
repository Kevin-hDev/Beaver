import type {
  PersistedToolResultMeta,
  ToolErrorInfo,
  ToolResultStatus,
} from "@/types/agent-tool-result";

interface SavedToolResult {
  name: string;
  result?: string;
  is_error?: boolean;
  result_meta?: PersistedToolResultMeta;
}

export function cancelledToolError(): ToolErrorInfo {
  return {
    code: "tool_cancelled",
    category: "cancelled",
    retryable: false,
  };
}

export function toolResultForModel(tool: SavedToolResult): string {
  const meta = tool.result_meta;
  const status = meta?.status ?? legacyStatus(tool.is_error);
  const error = meta?.error ?? legacyError(tool.is_error);
  const warnings = meta?.warnings ?? [];
  const truncated = meta?.truncated ?? false;
  const output = tool.result ?? "";

  if (status === "success" && !error && warnings.length === 0 && !truncated) {
    return output;
  }
  return JSON.stringify({
    kind: "tool_result",
    tool: tool.name,
    status,
    outputFormat: "raw_following",
    ...(error ? { error } : {}),
    ...(warnings.length > 0 ? { warnings } : {}),
    ...(truncated ? { truncated: true } : {}),
  }) + `\n${output}`;
}

function legacyStatus(isError?: boolean): ToolResultStatus {
  return isError ? "error" : "success";
}

function legacyError(isError?: boolean): ToolErrorInfo | undefined {
  return isError ? {
    code: "legacy_tool_error",
    category: "execution",
    retryable: false,
  } : undefined;
}
