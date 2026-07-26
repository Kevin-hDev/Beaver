const HIDDEN_AGENT_TOOLS = new Set([
  "todo_write",
  "todo_history",
  "todo_pause",
  "todo_resume",
  "todo_delete",
  "agent_diagnostics",
  "ask_user_choice",
  "planmode",
  "exitplanmode",
]);

export function isHiddenAgentTool(name: string) {
  return HIDDEN_AGENT_TOOLS.has(name);
}
