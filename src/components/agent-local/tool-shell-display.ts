import type { SavedSegment } from "@/types/agent";

interface ShellDisplayTool {
  name: string;
  summary: string;
  args?: Record<string, unknown>;
  result?: string;
}

export function isShellStopAction(tool: ShellDisplayTool): boolean {
  if (tool.name !== "bash_write") return false;
  if (tool.args?.stop === true) return true;
  return typeof tool.args?.chars === "string" && tool.args.chars.includes("\u0003");
}

export function isLegacyShellStopError(
  tool: ShellDisplayTool,
  isError: boolean | undefined,
): boolean {
  return isError === true
    && isShellStopAction(tool)
    && tool.result?.trim() === "Commande annulee.";
}

export function shellCommandPreview(
  tool: ShellDisplayTool,
  previousTools: ShellDisplayTool[],
): string | undefined {
  if (tool.name === "bash") return tool.summary;
  if (!isShellStopAction(tool)) return undefined;

  const sessionId = typeof tool.args?.session_id === "string" ? tool.args.session_id : "";
  if (tool.summary && tool.summary !== sessionId) return tool.summary;
  if (!sessionId) return undefined;

  return findShellCommand(previousTools, sessionId);
}

export function recoverLegacyShellStopSummaries(
  segments: SavedSegment[],
): SavedSegment[] {
  let changed = false;
  const recovered = segments.map((segment, segmentIndex) => {
    let segmentChanged = false;
    const tools = segment.tools.map((tool, toolIndex) => {
      const sessionId = legacyStopSessionId(tool);
      if (!sessionId) return tool;
      const command = findEarlierShellCommand(segments, segmentIndex, toolIndex, sessionId);
      if (!command) return tool;
      changed = true;
      segmentChanged = true;
      return { ...tool, summary: command };
    });
    return segmentChanged ? { ...segment, tools } : segment;
  });
  return changed ? recovered : segments;
}

function legacyStopSessionId(tool: ShellDisplayTool): string | undefined {
  if (!isShellStopAction(tool)) return undefined;
  const sessionId = typeof tool.args?.session_id === "string" ? tool.args.session_id : "";
  if (!/^[a-zA-Z0-9-]{1,128}$/.test(sessionId)) return undefined;
  return tool.summary === sessionId ? sessionId : undefined;
}

function findEarlierShellCommand(
  segments: SavedSegment[],
  beforeSegment: number,
  beforeTool: number,
  sessionId: string,
): string | undefined {
  const current = findShellCommand(segments[beforeSegment].tools, sessionId, beforeTool);
  if (current) return current;
  for (let segmentIndex = beforeSegment - 1; segmentIndex >= 0; segmentIndex -= 1) {
    const command = findShellCommand(segments[segmentIndex].tools, sessionId);
    if (command) return command;
  }
  return undefined;
}

function findShellCommand(
  tools: readonly ShellDisplayTool[],
  sessionId: string,
  beforeIndex = tools.length,
): string | undefined {
  for (let index = beforeIndex - 1; index >= 0; index -= 1) {
    const candidate = tools[index];
    if (candidate.name === "bash" && candidate.result?.includes(`session_id=${sessionId},`)) {
      return candidate.summary;
    }
  }
  return undefined;
}
