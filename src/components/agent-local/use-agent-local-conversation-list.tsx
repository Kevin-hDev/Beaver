import { useMemo } from "react";
import type { useAgentLocalTab } from "@/hooks/use-agent-local-tab";
import { ConversationList } from "./conversation-list";

export function useAgentLocalConversationList(
  state: ReturnType<typeof useAgentLocalTab>,
  activeSessionId: string | null,
) {
  const {
    sessions,
    projectsHook,
    rename,
    sessionActions,
    handleSelectById,
    handleDeleteProject,
    handleDeleteSession,
  } = state;
  const {
    handleCreate,
    handleCreateInProject,
  } = sessionActions;

  return useMemo(() => (
    <ConversationList
      sessions={sessions}
      projects={projectsHook.projects}
      selectedId={activeSessionId}
      onSelect={(id) => void handleSelectById(id)}
      onCreate={handleCreate}
      onRename={(id, name) => void rename(id, name)}
      onDelete={(id) => void handleDeleteSession(id)}
      onNewSessionInProject={(id) => void handleCreateInProject(id)}
      onRenameProject={(id, name) => void projectsHook.rename(id, name)}
      onDeleteProject={handleDeleteProject}
      onOpenFolder={(path) => void projectsHook.openFolder(path)}
      onReorderProjects={(ids) => void projectsHook.reorder(ids)}
    />
  ), [
    activeSessionId, handleCreate, handleCreateInProject, handleDeleteProject,
    handleDeleteSession, handleSelectById, projectsHook, rename, sessions,
  ]);
}
