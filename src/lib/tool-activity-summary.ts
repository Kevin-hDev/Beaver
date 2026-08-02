export type ToolActivityGroupKind =
  | "exploration"
  | "modification"
  | "command"
  | "web"
  | "git"
  | "memory"
  | "other";

const GROUP_ICONS: Record<ToolActivityGroupKind, string> = {
  exploration: "Compass",
  modification: "PencilSimple",
  command: "TerminalWindow",
  web: "Globe",
  git: "GitBranch",
  memory: "BookOpenText",
  other: "Wrench",
};

export function groupIcon(kind: ToolActivityGroupKind): string {
  return GROUP_ICONS[kind];
}

export interface ToolActivitySummaryInput {
  name: string;
  result?: string;
  is_error?: boolean;
  isError?: boolean;
  domain?: "memory";
}

export interface ToolActivityCounts {
  files: number;
  searches: number;
  lists: number;
  writes: number;
  edits: number;
  commands: number;
  webSearches: number;
  webFetches: number;
  gitActions: number;
  otherActions: number;
}

export interface ToolActivityGroup<T extends ToolActivitySummaryInput> {
  kind: ToolActivityGroupKind;
  tools: T[];
  counts: ToolActivityCounts;
  isPending: boolean;
  hasError: boolean;
}

const READ_TOOLS = new Set([
  "read_file", "read_spreadsheet", "read_document",
]);
const SEARCH_TOOLS = new Set(["grep", "glob"]);
const WRITE_TOOLS = new Set([
  "write_file", "write_spreadsheet", "write_document", "transform_image",
]);
const EDIT_TOOLS = new Set(["edit_file"]);
const WEB_TOOLS = new Set(["web_search", "web_fetch"]);
const GIT_TOOLS = new Set(["create_branch", "checkout_branch"]);

function emptyCounts(): ToolActivityCounts {
  return {
    files: 0,
    searches: 0,
    lists: 0,
    writes: 0,
    edits: 0,
    commands: 0,
    webSearches: 0,
    webFetches: 0,
    gitActions: 0,
    otherActions: 0,
  };
}

function getToolActivityKind(name: string, domain?: "memory"): ToolActivityGroupKind {
  if (domain === "memory") return "memory";
  if (READ_TOOLS.has(name) || SEARCH_TOOLS.has(name) || name === "list_dir") {
    return "exploration";
  }
  if (WRITE_TOOLS.has(name) || EDIT_TOOLS.has(name)) return "modification";
  if (name === "bash" || name === "bash_control") return "command";
  if (WEB_TOOLS.has(name)) return "web";
  if (GIT_TOOLS.has(name)) return "git";
  return "other";
}

function hasToolResult(tool: ToolActivitySummaryInput): boolean {
  return tool.result !== undefined || tool.is_error !== undefined || tool.isError !== undefined;
}

function toolHasError(tool: ToolActivitySummaryInput): boolean {
  return tool.is_error === true || tool.isError === true;
}

export function groupToolActivities<T extends ToolActivitySummaryInput>(
  tools: T[],
): ToolActivityGroup<T>[] {
  const groups: ToolActivityGroup<T>[] = [];
  for (const tool of tools) {
    const kind = getToolActivityKind(tool.name, tool.domain);
    let group = groups.find((candidate) => candidate.kind === kind);
    if (!group) {
      group = {
        kind,
        tools: [],
        counts: emptyCounts(),
        isPending: false,
        hasError: false,
      };
      groups.push(group);
    }
    group.tools.push(tool);
    group.isPending = group.isPending || !hasToolResult(tool);
    group.hasError = group.hasError || toolHasError(tool);
    incrementCounts(group.counts, tool.name);
  }
  return groups;
}

function incrementCounts(counts: ToolActivityCounts, name: string) {
  if (READ_TOOLS.has(name)) counts.files += 1;
  else if (SEARCH_TOOLS.has(name)) counts.searches += 1;
  else if (name === "list_dir") counts.lists += 1;
  else if (WRITE_TOOLS.has(name)) counts.writes += 1;
  else if (EDIT_TOOLS.has(name)) counts.edits += 1;
  else if (name === "bash" || name === "bash_control") counts.commands += 1;
  else if (name === "web_search") counts.webSearches += 1;
  else if (name === "web_fetch") counts.webFetches += 1;
  else if (GIT_TOOLS.has(name)) counts.gitActions += 1;
  else counts.otherActions += 1;
}
