import type { AgentSessionMeta, Project } from "@/types/agent";

export interface ConversationListProps {
  sessions: AgentSessionMeta[];
  projects: Project[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onCreate: () => void;
  onRename: (id: string, name: string) => void;
  onDelete: (id: string) => void;
  onNewSessionInProject: (projectId: string) => void;
  onRenameProject: (id: string, name: string) => void;
  onDeleteProject: (id: string) => void;
  onOpenFolder: (path: string) => void;
  onReorderProjects: (ids: string[]) => void;
}
