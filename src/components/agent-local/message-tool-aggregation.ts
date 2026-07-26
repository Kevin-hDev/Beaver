import type { ToolActivityRecord } from "@/types/agent";

export function extractBranchActivity(
  tools: ToolActivityRecord[],
): { action: "created" | "switched"; branchName: string; path?: string } | null {
  for (const tool of tools) {
    if (tool.name === "create_branch" && tool.result && !tool.is_error) {
      return { action: "created", branchName: tool.summary, path: tool.result };
    }
    if (tool.name === "checkout_branch" && tool.result && !tool.is_error) {
      return { action: "switched", branchName: tool.summary };
    }
  }
  return null;
}
