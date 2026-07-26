import type { SubagentInfo } from "@/types/agent";

type SubagentType = SubagentInfo["type"];

const LEGACY_NAMES = new Set(["", "agent", "explore", "explorer", "coder"]);

function subagentDefaultName(type: SubagentType): string {
  return type === "coder" ? "Claudiator" : "Geminitor";
}

export function subagentDisplayName(agent: Pick<SubagentInfo, "name" | "type">): string {
  return subagentDefaultName(agent.type);
}

export function subagentColorKey(
  agent: Pick<SubagentInfo, "type" | "colorKey">,
): string {
  return agent.colorKey || (agent.type === "coder" ? "claudiator" : "geminitor");
}

export function subagentSecondaryText(agent: SubagentInfo): string {
  return agent.description
    || legacyMissionName(agent)
    || agent.lastActivity?.label
    || agent.summary
    || agent.promptPreview
    || "...";
}

function legacyMissionName(agent: Pick<SubagentInfo, "name" | "type">): string | undefined {
  const name = agent.name.trim();
  if (!name) return undefined;
  if (LEGACY_NAMES.has(name.toLowerCase())) return undefined;
  return name.toLowerCase() === subagentDefaultName(agent.type).toLowerCase()
    ? undefined
    : name;
}
