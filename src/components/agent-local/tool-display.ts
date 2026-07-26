import type { TFunction } from "i18next";
import type { RenderableTool } from "./tool-detail-row";

export interface ToolDisplayInfo {
  label: string;
  summary: string;
  additions?: number;
  deletions?: number;
  icon: string;
  /** Partie dossier du chemin (tronçonable). Vide si pas un chemin de fichier. */
  dir?: string;
  /** Nom du fichier seul (toujours visible à droite). Vide si pas un chemin de fichier. */
  fileName?: string;
}

const FILE_TOOLS = new Set([
  "read_file", "write_file", "edit_file", "read_spreadsheet", "read_document",
  "read_image", "write_spreadsheet", "write_document", "process_image",
]);

/** Nombre maximum de dossiers parents affichés avant le nom du fichier. */
const MAX_PARENT_DIRS = 3;

/**
 * Coupe un chemin pour ne garder que les `maxDirs` derniers dossiers parents
 * + le nom du fichier. Si on coupe, on préfixe avec "…/".
 * "a/b/c/d/e/file.ts" avec maxDirs=3 -> "…/c/d/e/file.ts"
 */
function trimToParentDirs(path: string, maxDirs: number): string {
  const parts = path.split("/").filter(Boolean);
  if (parts.length <= maxDirs + 1) return path;
  const kept = parts.slice(parts.length - maxDirs - 1);
  return `…/${kept.join("/")}`;
}

/**
 * Sépare un chemin en deux segments : la partie dossier (tronçable) et le nom
 * du fichier (fixe, toujours visible). Sert au layout flex qui tronque uniquement
 * la partie gauche (dossiers).
 */
export function filePathSegments(path: string, projectPath?: string): {
  dirs: string;
  fileName: string;
} {
  const short = shortenPath(path, projectPath);
  const parts = short.split("/").filter(Boolean);
  if (parts.length === 0) return { dirs: "", fileName: "" };
  if (parts.length === 1) return { dirs: "", fileName: parts[0] };
  const fileName = parts[parts.length - 1];
  const dirs = parts.slice(0, -1).join("/");
  return { dirs: dirs ? `${dirs}/` : "", fileName };
}

const FILE_ICONS: Record<string, string> = {
  read_file: "BookOpenText",
  read_spreadsheet: "FileText",
  read_document: "FileText",
  read_image: "Image",
  write_file: "FilePlus",
  write_spreadsheet: "FilePlus",
  write_document: "FilePlus",
  edit_file: "Pencil",
  process_image: "Pencil",
  bash: "TerminalWindow",
  web_search: "Globe",
  web_fetch: "Link",
  list_dir: "FolderOpen",
  grep: "MagnifyingGlass",
  glob: "MagnifyingGlass",
  create_branch: "GitBranch",
  checkout_branch: "GitBranch",
  load_skill: "Sparkle",
  delegate_task: "Users",
  forecast_data_audit: "ChartLineUp",
  forecast: "ChartLineUp",
  forecast_models: "ChartLineUp",
  forecast_read: "ChartLineUp",
  forecast_analyze: "ChartLineUp",
  search_mcp_tools: "Plugs",
};

function iconFor(name: string): string {
  return FILE_ICONS[name] ?? "Wrench";
}

export function toolDisplayInfo(
  tool: RenderableTool,
  projectPath: string | undefined,
  t: TFunction,
): ToolDisplayInfo {
  if (tool.name === "bash") {
    return { label: tool.name, summary: compactCommand(tool.summary), icon: iconFor(tool.name) };
  }
  if (tool.name === "web_search" || tool.name === "web_fetch") {
    return { label: tool.name, summary: tool.summary, icon: iconFor(tool.name) };
  }
  const labelKey = actionKey(tool.name);
  const summary = displaySummary(tool, projectPath);
  const isFilePath = FILE_TOOLS.has(tool.name);
  // Priorité au chemin résolu côté backend (toujours absolu), sinon fallback sur le summary.
  const pathForDisplay = tool.resolved_path || tool.summary;
  const segments = isFilePath ? filePathSegments(pathForDisplay, projectPath) : null;
  return {
    label: t(`agentLocal.toolActivity.actions.${labelKey}`),
    summary,
    ...changeStats(tool),
    icon: iconFor(tool.name),
    dir: segments?.dirs,
    fileName: segments?.fileName,
  };
}

function actionKey(name: string): string {
  if (name === "load_skill") return "skill";
  if (name === "delegate_task") return "agent";
  if (name === "forecast" || name === "forecast_models" || name === "forecast_read") return "read";
  if (name === "forecast_data_audit" || name === "forecast_analyze") return "forecast";
  if (name === "search_mcp_tools") return "mcp";
  if (name === "list_dir") return "list";
  if (name === "grep" || name === "glob") return "search";
  if (name.startsWith("read_")) return "read";
  if (name.startsWith("write_")) return "create";
  if (name === "edit_file" || name === "process_image") return "edit";
  if (name === "create_branch") return "createBranch";
  if (name === "checkout_branch") return "switchBranch";
  return "tool";
}

function displaySummary(tool: RenderableTool, projectPath?: string): string {
  if (!FILE_TOOLS.has(tool.name) && tool.name !== "list_dir") return tool.summary;
  return shortenPath(tool.summary, projectPath);
}

/**
 * Raccourcit un chemin en gardant au maximum MAX_PARENT_DIRS dossiers parents
 * + le nom du fichier. Si le chemin fait partie du projet, on repart de la racine
 * du projet (rootName + chemin relatif). Sinon on retire le préfixe utilisateur.
 */
export function shortenPath(path: string, projectPath?: string): string {
  const normalized = path.replace(/\\/g, "/");
  const normalizedProject = projectPath?.replace(/\\/g, "/").replace(/\/+$/, "");
  let displayPath = normalized;
  if (normalizedProject && isInsideProject(normalized, normalizedProject)) {
    const rootName = basename(normalizedProject);
    const relative = normalized.slice(normalizedProject.length).replace(/^\/+/, "");
    displayPath = relative ? `${rootName}/${relative}` : rootName;
  } else {
    const projectsMarker = "/Projects/";
    const markerIndex = normalized.indexOf(projectsMarker);
    if (markerIndex >= 0) {
      displayPath = normalized.slice(markerIndex + projectsMarker.length);
    } else {
      displayPath = normalized.replace(/^\/Users\/[^/]+\//, "");
    }
  }
  return trimToParentDirs(displayPath, MAX_PARENT_DIRS);
}

function isInsideProject(path: string, projectPath: string): boolean {
  return path === projectPath || path.startsWith(`${projectPath}/`);
}

function basename(path: string): string {
  const parts = path.split("/").filter(Boolean);
  return parts.length > 0 ? parts[parts.length - 1] : path;
}

function changeStats(tool: RenderableTool): Pick<ToolDisplayInfo, "additions" | "deletions"> {
  if (tool.name === "write_file" && tool.content != null) {
    return { additions: countLines(tool.content), deletions: 0 };
  }
  if (tool.name === "edit_file" && tool.old_text != null && tool.new_text != null) {
    return { additions: countLines(tool.new_text), deletions: countLines(tool.old_text) };
  }
  return {};
}

function countLines(text: string): number {
  if (text.length === 0) return 0;
  return text.replace(/\n$/, "").split("\n").length;
}

function compactCommand(command: string): string {
  const firstLine = command.split(/\r?\n/, 1)[0] ?? "";
  const maxLength = 96;
  if (command.includes("\n") || command.includes("\r")) return `${firstLine.slice(0, maxLength)}...`;
  if (firstLine.length > maxLength) return `${firstLine.slice(0, maxLength)}...`;
  return firstLine;
}
