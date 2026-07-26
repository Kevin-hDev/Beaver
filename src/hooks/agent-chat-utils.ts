import type { ToolActivityRecord, ToolFileChangeRecord, TokenPhase } from "@/types/agent";

export interface ToolActivity {
  name: string;
  args: Record<string, unknown>;
  domain?: "memory";
  result?: string;
  isError?: boolean;
  /** Résumé lisible fourni par le backend, séparé des arguments techniques. */
  displaySummary?: string;
  /** Chemin absolu résolu côté backend (working_dir + path). Utilisé pour l'affichage. */
  resolvedPath?: string;
  /** Fichiers touchés indirectement par l'outil, notamment via bash. */
  affectedPaths?: string[];
  /** Diff historique figé au moment où l'outil a modifié le fichier. */
  fileChanges?: ToolFileChangeRecord[];
  /** Ligne de départ structurée d'une modification de fichier. */
  startLine?: number;
}

export interface StreamSegment {
  thinking: string;
  tools: ToolActivity[];
  content: string;
  phase?: TokenPhase;
}

function str(v: unknown, fallback = ""): string {
  return typeof v === "string" ? v : fallback;
}

export function toolsToRecords(tools: ToolActivity[]): ToolActivityRecord[] {
  return tools.map((t) => {
    const a = t.args;
    let summary = "";
    if (t.displaySummary !== undefined) summary = t.displaySummary;
    else if (t.name === "load_skill") summary = "";
    else if (t.name === "bash") summary = str(a.command);
    else if (t.name === "read_file" || t.name === "write_file") summary = str(a.path);
    else if (t.name === "edit_file") summary = str(a.path);
    else if (t.name === "list_dir") summary = str(a.path, ".");
    else if (t.name === "grep") summary = str(a.pattern);
    else if (t.name === "glob") summary = str(a.pattern);
    else if (t.name === "web_search") summary = str(a.query);
    else if (t.name === "web_fetch") summary = str(a.url);
    else if (t.name === "read_spreadsheet") summary = str(a.path);
    else if (t.name === "read_document") summary = str(a.path);
    else if (t.name === "read_image") summary = str(a.path);
    else if (t.name === "write_spreadsheet") summary = str(a.path);
    else if (t.name === "write_document") summary = str(a.path);
    else if (t.name === "process_image") summary = str(a.input_path);
    else summary = JSON.stringify(a).slice(0, 80);

    return {
      name: t.name,
      summary,
      domain: t.domain,
      args: a,
      result: t.result,
      is_error: t.isError,
      resolved_path: t.resolvedPath,
      affected_paths: t.affectedPaths,
      file_changes: t.fileChanges,
      content: t.name === "write_file" ? str(a.content)
        : t.name === "write_document" ? JSON.stringify(Array.isArray(a.content) ? a.content : [])
        : t.name === "write_spreadsheet" ? JSON.stringify(Array.isArray(a.operations) ? a.operations : [])
        : undefined,
      old_text: t.name === "edit_file" ? str(a.old_string) : undefined,
      new_text: t.name === "edit_file" ? str(a.new_string) : undefined,
      start_line: t.startLine,
    };
  });
}

export function segmentsToRecords(segments: StreamSegment[]): ToolActivityRecord[] {
  return segments.flatMap((seg) => toolsToRecords(seg.tools));
}

export function buildSegmentedMessage(
  allSegments: StreamSegment[],
): { content: string; thinking?: string; toolRecords?: ToolActivityRecord[]; segments?: SavedSegment[] } {
  const fullContent = allSegments.map((seg) => seg.content).filter(Boolean).join("\n\n");
  const fullThinking = allSegments.map((seg) => seg.thinking).filter(Boolean).join("\n\n");
  const allToolRecords = segmentsToRecords(allSegments);

  return {
    content: fullContent,
    thinking: fullThinking || undefined,
    toolRecords: allToolRecords.length > 0 ? allToolRecords : undefined,
    segments: allSegments.map((seg) => ({
      thinking: seg.thinking || undefined,
      tools: toolsToRecords(seg.tools),
      content: seg.content,
      phase: seg.phase,
    })),
  };
}

export interface SavedSegment {
  thinking?: string;
  tools: ToolActivityRecord[];
  content: string;
  phase?: TokenPhase;
}

export interface ChatMsg {
  role: string;
  content: string;
  images?: string[] | null;
  tool_calls?: Array<{
    id?: string;
    function: { name: string; arguments: Record<string, unknown> };
  }> | null;
  tool_name?: string | null;
  tool_call_id?: string | null;
}

function rebuildArgs(name: string, summary: string): Record<string, string> {
  if (name === "web_search") return { query: summary };
  if (name === "web_fetch") return { url: summary };
  if (name === "bash") return { command: summary };
  if (name === "grep" || name === "glob") return { pattern: summary };
  if (["read_file", "write_file", "edit_file", "list_dir"].includes(name)) return { path: summary };
  if (["read_spreadsheet", "read_document", "read_image", "write_spreadsheet", "write_document"].includes(name)) return { path: summary };
  if (name === "process_image") return { input_path: summary };
  return { input: summary };
}

export function restoredToolArguments(
  activity: ToolActivityRecord,
): Record<string, unknown> {
  return activity.args ?? rebuildArgs(activity.name, activity.summary);
}

export function expandToolActivities(
  activities: ToolActivityRecord[], content: string,
): ChatMsg[] {
  return expandTurnFlat(activities, content);
}

export function expandSegmentsToChat(
  segments: SavedSegment[], fallbackContent: string,
): ChatMsg[] {
  const msgs: ChatMsg[] = [];
  let idCounter = 0;
  for (const seg of segments) {
    if (seg.tools.length > 0) {
      const toolCalls = seg.tools.map((t) => {
        const id = `restored-${idCounter++}`;
        return { id, function: { name: t.name, arguments: restoredToolArguments(t) } };
      });
      msgs.push({ role: "assistant", content: seg.content || "", tool_calls: toolCalls });
      for (const tc of toolCalls) {
        const tool = seg.tools[toolCalls.indexOf(tc)];
        msgs.push({ role: "tool", content: tool.result ?? "", tool_name: tool.name, tool_call_id: tc.id });
      }
    } else if (seg.content) {
      msgs.push({ role: "assistant", content: seg.content });
    }
  }
  if (msgs.length === 0 && fallbackContent) {
    msgs.push({ role: "assistant", content: fallbackContent });
  }
  return msgs;
}

function expandTurnFlat(activities: ToolActivityRecord[], content: string): ChatMsg[] {
  const msgs: ChatMsg[] = [];
  const toolCalls = activities.map((t, i) => ({
    id: `restored-${i}`,
    function: { name: t.name, arguments: restoredToolArguments(t) },
  }));
  msgs.push({ role: "assistant", content: "", tool_calls: toolCalls });
  for (let i = 0; i < activities.length; i++) {
    const t = activities[i];
    msgs.push({
      role: "tool", content: t.result ?? "",
      tool_name: t.name, tool_call_id: `restored-${i}`,
    });
  }
  if (content) {
    msgs.push({ role: "assistant", content });
  }
  return msgs;
}
