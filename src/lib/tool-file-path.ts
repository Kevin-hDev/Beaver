import type { ToolActivityRecord } from "@/types/agent";

const FILE_TOOLS = new Set(["read_file", "write_file", "edit_file", "read_spreadsheet", "read_document", "read_image", "write_spreadsheet", "write_document", "process_image"]);
const PATH_KEYS = ["path", "file_path", "filepath", "target_path"];

export function isFileTool(name: string): boolean {
  return FILE_TOOLS.has(name);
}

export function extractToolPath(args: Record<string, unknown>): string {
  for (const key of PATH_KEYS) {
    const value = args[key];
    if (typeof value === "string" && value.trim()) return value;
  }
  return "";
}

export function inferSavedToolPaths(
  tools: ToolActivityRecord[],
  initialPath = "",
): ToolActivityRecord[] {
  let lastPath = initialPath;
  return tools.map((tool) => {
    if (!isFileTool(tool.name)) return tool;
    const summary = tool.summary.trim() || lastPath;
    if (summary) lastPath = summary;
    return summary === tool.summary ? tool : { ...tool, summary };
  });
}

export function lastSavedToolPath(tools: ToolActivityRecord[], initialPath = ""): string {
  let lastPath = initialPath;
  for (const tool of tools) {
    if (isFileTool(tool.name) && tool.summary.trim()) lastPath = tool.summary;
  }
  return lastPath;
}
