import type { AgentSessionMeta, Project } from "@/types/agent";
import type { DirectoryAccessPromptProps } from "./directory-access-prompt";

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
  /* Ouvre le sélecteur de dossier puis enregistre le projet choisi. */
  onAddProject: () => void;
  onReorderProjects: (ids: string[]) => void;
  /* Range une liste de conversations. `projectId` nul désigne celles qui
     n'appartiennent à aucun projet. */
  onReorderSessions: (projectId: string | null, ids: string[]) => void;
  /* Range la section « Épinglé ». Les épinglées ont quitté leur liste d'origine. */
  onReorderPinnedSessions: (ids: string[]) => void;
  /* Épingle ou désépingle selon l'état courant de la conversation. */
  onTogglePin: (id: string) => void;
  directoryAccessPrompt?: DirectoryAccessPromptProps;
}
