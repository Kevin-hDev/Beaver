import type { ToolErrorInfo } from "@/types/agent-tool-result";

export function cancelledToolError(): ToolErrorInfo {
  return {
    code: "tool_cancelled",
    category: "cancelled",
    retryable: false,
  };
}
