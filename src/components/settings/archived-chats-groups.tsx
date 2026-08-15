import { FolderStateIcon } from "@/components/ui/folder-state-icon";
import { SessionIcon } from "@/components/ui/session-icon";
import type { AgentSessionMeta, Project } from "@/types/agent";
import type { SelectOption } from "./settings-select";

export const DISCUSSIONS_FILTER = "__discussions__";
export const ALL_FILTER = "__all__";

export interface ArchiveGroup {
  id: string;
  title: string;
  kind: "project" | "discussions";
  sessions: AgentSessionMeta[];
}

export function buildArchiveGroups(
  sessions: AgentSessionMeta[],
  projects: Project[],
  projectMap: Map<string, Project>,
  query: string,
  filter: string,
  discussionsTitle: string,
): ArchiveGroup[] {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const matches = (session: AgentSessionMeta) =>
    !normalizedQuery || session.name.toLocaleLowerCase().includes(normalizedQuery);
  const sorted = [...sessions.filter(matches)].sort((a, b) => activityTime(b) - activityTime(a));
  const groups: ArchiveGroup[] = [];
  for (const project of projects) {
    if (filter !== ALL_FILTER && filter !== project.id) continue;
    const projectSessions = sorted.filter((session) => session.project_id === project.id);
    if (projectSessions.length > 0) groups.push({ id: project.id, title: project.name, kind: "project", sessions: projectSessions });
  }
  if (filter === ALL_FILTER || filter === DISCUSSIONS_FILTER) {
    const discussions = sorted.filter((session) => !session.project_id || !projectMap.has(session.project_id));
    if (discussions.length > 0) groups.push({ id: DISCUSSIONS_FILTER, title: discussionsTitle, kind: "discussions", sessions: discussions });
  }
  return groups;
}

export function projectFilterOptions(t: (key: string) => string, projects: Project[]): SelectOption[] {
  return [
    { value: ALL_FILTER, label: t("settings.archivedChats.allProjects"), icon: <FolderStateIcon open={false} size="var(--icon-sm)" /> },
    { value: DISCUSSIONS_FILTER, label: t("projects.discussions"), icon: <SessionIcon size="var(--icon-sm)" /> },
    ...projects.map((project) => ({
      value: project.id,
      label: project.name,
      icon: <FolderStateIcon open={false} size="var(--icon-sm)" />,
    })),
  ];
}

function activityTime(session: AgentSessionMeta): number {
  return new Date(session.updated_at ?? session.created_at).getTime();
}
