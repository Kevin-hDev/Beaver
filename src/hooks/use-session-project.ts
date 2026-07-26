import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import type { AgentSession, Project } from "@/types/agent";
import { AGENT_SESSIONS_CHANGED } from "./agent-session-events";

const SESSION_DIRECTORY_ID = "session-working-directory";

function sessionDirectory(path: string): Project {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return {
    id: SESSION_DIRECTORY_ID,
    name: parts[parts.length - 1] ?? path,
    path,
    order: 0,
    created_at: "",
  };
}

export function useSessionProject(
  sessionId: string,
  projects: Project[],
  onAddProject: (path: string) => Promise<Project>,
  hasMessages: boolean,
) {
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [workingDir, setWorkingDir] = useState("");
  const [loading, setLoading] = useState(true);

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      const session = await invoke<AgentSession>("get_agent_session", { id: sessionId });
      setSelectedProjectId(session.project_id ?? null);
      setWorkingDir(session.working_dir ?? "");
    } catch {
      setSelectedProjectId(null);
      setWorkingDir("");
    } finally {
      setLoading(false);
    }
  }, [sessionId]);

  useEffect(() => {
    queueMicrotask(() => void reload());
    const refresh = () => void reload();
    window.addEventListener(AGENT_SESSIONS_CHANGED, refresh);
    return () => window.removeEventListener(AGENT_SESSIONS_CHANGED, refresh);
  }, [reload]);

  const savedProject = projects.find((project) => project.id === selectedProjectId);
  const selectedProject = savedProject
    ?? (workingDir ? sessionDirectory(workingDir) : undefined);
  const locked = hasMessages && !!selectedProject;
  const hidden = loading || (hasMessages && !selectedProject);

  const handleAddProject = useCallback(async () => {
    const result = await openFileDialog({ directory: true });
    if (!result) return;
    const path = typeof result === "string" ? result : String(result);
    const project = await onAddProject(path);
    setSelectedProjectId(project.id);
  }, [onAddProject]);

  return {
    selectedProjectId,
    setSelectedProjectId,
    selectedProject,
    workingDir,
    locked,
    hidden,
    handleAddProject,
  };
}
