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
    reorderSessions,
    reorderPinned,
    togglePin,
    sessionActions,
    handleSelectById,
    handleDeleteProject,
    handleDeleteSession,
  } = state;
  const {
    handleCreate,
    handleCreateInProject,
    handleAddProject,
    directoryAccessPrompt,
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
      onDeleteProject={(id) => void handleDeleteProject(id)}
      onOpenFolder={(path) => void projectsHook.openFolder(path)}
      onAddProject={() => void handleAddProject()}
      onReorderProjects={(ids) => void projectsHook.reorder(ids)}
      onReorderSessions={(projectId, ids) => void reorderSessions(projectId, ids)}
      onReorderPinnedSessions={(ids) => void reorderPinned(ids)}
      onTogglePin={(id) => void togglePin(id)}
      directoryAccessPrompt={directoryAccessPrompt}
    />
  ), [
    activeSessionId, handleAddProject, handleCreate, handleCreateInProject, handleDeleteProject,
    handleDeleteSession, handleSelectById, projectsHook, rename, reorderSessions, sessions,
    reorderPinned, togglePin, directoryAccessPrompt,
  ]);
}
