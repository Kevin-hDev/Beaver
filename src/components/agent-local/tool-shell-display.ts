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

  for (let index = previousTools.length - 1; index >= 0; index -= 1) {
    const candidate = previousTools[index];
    if (
      candidate.name === "bash"
      && candidate.result?.includes(`session_id=${sessionId},`)
    ) {
      return candidate.summary;
    }
  }
  return undefined;
}
