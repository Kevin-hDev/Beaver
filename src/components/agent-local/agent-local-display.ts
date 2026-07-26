import type { AgentSessionMeta, Project } from "@/types/agent";

export function resolveDisplaySession(
  sessions: AgentSessionMeta[],
  displaySessionId: string | null,
  activeSession: AgentSessionMeta | null | undefined,
): AgentSessionMeta | null {
  if (!displaySessionId) return null;
  return sessions.find((session) => session.id === displaySessionId) ?? activeSession ?? null;
}

export function resolveDisplayProject(
  projects: Project[],
  displaySession: AgentSessionMeta | null,
  activeProject: Project | null | undefined,
): Project | null {
  if (!displaySession?.project_id) return activeProject ?? null;
  return projects.find((project) => project.id === displaySession.project_id) ?? activeProject ?? null;
}

export function resolveDisplayReasoningMode(
  displaySession: AgentSessionMeta | null,
  defaultMode: string | null,
): string | null {
  if (!displaySession) return defaultMode;
  return displaySession.reasoning_mode ?? (displaySession.thinking_enabled ? "auto" : null);
}

export function resolveDisplayModel(
  displaySession: AgentSessionMeta | null,
  defaultModel: string,
  defaultProvider: string,
) {
  return {
    displayModel: displaySession?.model ?? defaultModel,
    displayProvider: displaySession?.provider ?? defaultProvider,
  };
}
