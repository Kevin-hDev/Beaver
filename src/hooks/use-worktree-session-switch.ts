import { useCallback, useState } from "react";
import type { Project } from "@/types/agent";

export interface WorktreeSwitchTarget {
  path: string;
  branch: string;
}

interface UseWorktreeSessionSwitchDeps {
  projects: Project[];
  model: string;
  provider: string;
  onAddProject: (path: string) => Promise<Project>;
  onNewSessionInProject?: (model: string, provider: string, projectId: string) => void;
}

export function normalizeProjectPath(path: string) {
  let normalized = path.trim();
  while (
    normalized.length > 1
    && /[\\/]$/.test(normalized)
    && !/^[A-Za-z]:[\\/]$/.test(normalized)
  ) {
    normalized = normalized.slice(0, -1);
  }
  return normalized;
}

export function findProjectByPath(projects: Project[], path: string) {
  const target = normalizeProjectPath(path);
  return projects.find((project) => normalizeProjectPath(project.path) === target);
}

export function useWorktreeSessionSwitch({
  projects,
  model,
  provider,
  onAddProject,
  onNewSessionInProject,
}: UseWorktreeSessionSwitchDeps) {
  const [pending, setPending] = useState<WorktreeSwitchTarget | null>(null);

  const request = useCallback((path: string, branch: string) => {
    setPending({ path, branch });
  }, []);

  const cancel = useCallback(() => {
    setPending(null);
  }, []);

  const createSession = useCallback(async () => {
    if (!pending || !onNewSessionInProject) return;
    try {
      const project = findProjectByPath(projects, pending.path) ?? await onAddProject(pending.path);
      onNewSessionInProject(model, provider, project.id);
      setPending(null);
    } catch (e) {
      console.error("worktree session switch:", e);
    }
  }, [model, onAddProject, onNewSessionInProject, pending, projects, provider]);

  return { pending, request, cancel, createSession };
}
