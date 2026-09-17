import type { ToolActivityRecord } from "@/types/agent";

const FILE_TOOLS = new Set(["read_file", "write_file", "edit_file", "read_spreadsheet", "read_document", "write_spreadsheet", "write_document", "transform_image"]);

export function isFileTool(name: string): boolean {
  return FILE_TOOLS.has(name);
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
