import type { AgentSessionMeta } from "@/types/agent";

const SESSION_GROUP_PREFIX = "session:";
const MAX_ANCESTORS = 128;

export function terminalWorkspaceGroupKey(
  session: AgentSessionMeta,
  sessions: AgentSessionMeta[],
  projectIds: string[],
): string {
  const byId = new Map(sessions.map((entry) => [entry.id, entry]));
  const projects = new Set(projectIds);
  const seen = new Set<string>();
  let current: AgentSessionMeta | undefined = session;

  for (let depth = 0; current && depth < MAX_ANCESTORS; depth += 1) {
    // Un projet retiré est un propriétaire mort : sa discussion redevient
    // autonome au lieu de conserver une clé que le backend ne peut plus ouvrir.
    if (current.project_id && projects.has(current.project_id)) return current.project_id;
    if (!seen.add(current.id)) break;
    const parentId = current.clone_root_session_id
      ?? current.clone_parent_session_id
      ?? current.parent_session_id;
    if (!parentId) return `${SESSION_GROUP_PREFIX}${current.id}`;
    current = byId.get(parentId);
  }

  // Repli UI uniquement : il empêche un état corrompu de partager un dock.
  // Rust reste l'autorité de sécurité et refuse la même ascendance invalide.
  return `${SESSION_GROUP_PREFIX}${session.id}`;
}

export function terminalWorkspaceGroupKeys(
  sessions: AgentSessionMeta[],
  projectIds: string[],
): string[] {
  const keys = new Set(projectIds);
  for (const session of sessions) {
    if (session.parent_session_id || session.clone_parent_session_id) continue;
    keys.add(terminalWorkspaceGroupKey(session, sessions, projectIds));
  }
  return [...keys];
}
