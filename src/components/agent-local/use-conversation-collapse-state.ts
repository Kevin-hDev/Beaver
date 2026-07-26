import { useCallback, useEffect, useMemo, useState } from "react";

const STORAGE_KEY = "clgo-conversation-collapse-v1";
const MAX_STORAGE_CHARS = 32_768;
const MAX_COLLAPSED_PROJECTS = 256;
const MAX_PROJECT_ID_CHARS = 128;
const PROJECT_ID_PATTERN = /^[a-zA-Z0-9-]+$/;

interface ConversationCollapseState {
  projectsCollapsed: boolean;
  discussionsCollapsed: boolean;
  collapsedProjects: Set<string>;
}

const DEFAULT_STATE: ConversationCollapseState = {
  projectsCollapsed: false,
  discussionsCollapsed: false,
  collapsedProjects: new Set(),
};

function validProjectId(value: unknown): value is string {
  return typeof value === "string"
    && value.length > 0
    && value.length <= MAX_PROJECT_ID_CHARS
    && PROJECT_ID_PATTERN.test(value);
}

function loadState(): ConversationCollapseState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw || raw.length > MAX_STORAGE_CHARS) return DEFAULT_STATE;
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return DEFAULT_STATE;

    const value = parsed as Record<string, unknown>;
    if (typeof value.projectsCollapsed !== "boolean"
      || typeof value.discussionsCollapsed !== "boolean"
      || !Array.isArray(value.collapsedProjectIds)) return DEFAULT_STATE;

    const ids = value.collapsedProjectIds
      .slice(0, MAX_COLLAPSED_PROJECTS)
      .filter(validProjectId);
    return {
      projectsCollapsed: value.projectsCollapsed,
      discussionsCollapsed: value.discussionsCollapsed,
      collapsedProjects: new Set(ids),
    };
  } catch {
    return DEFAULT_STATE;
  }
}

function saveState(state: ConversationCollapseState) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      projectsCollapsed: state.projectsCollapsed,
      discussionsCollapsed: state.discussionsCollapsed,
      collapsedProjectIds: [...state.collapsedProjects].slice(0, MAX_COLLAPSED_PROJECTS),
    }));
  } catch {
    // L'état courant reste utilisable même si le stockage local est indisponible.
  }
}

export function useConversationCollapseState() {
  const [state, setState] = useState(loadState);

  useEffect(() => saveState(state), [state]);

  const toggleProjects = useCallback(() => {
    setState((current) => ({ ...current, projectsCollapsed: !current.projectsCollapsed }));
  }, []);

  const toggleDiscussions = useCallback(() => {
    setState((current) => ({ ...current, discussionsCollapsed: !current.discussionsCollapsed }));
  }, []);

  const toggleProject = useCallback((id: string) => {
    if (!validProjectId(id)) return;
    setState((current) => {
      const collapsedProjects = new Set(current.collapsedProjects);
      if (collapsedProjects.has(id)) {
        collapsedProjects.delete(id);
      } else {
        if (collapsedProjects.size >= MAX_COLLAPSED_PROJECTS) {
          const oldest = collapsedProjects.values().next().value;
          if (typeof oldest === "string") collapsedProjects.delete(oldest);
        }
        collapsedProjects.add(id);
      }
      return { ...current, collapsedProjects };
    });
  }, []);

  return useMemo(() => ({
    ...state,
    toggleProjects,
    toggleDiscussions,
    toggleProject,
  }), [state, toggleDiscussions, toggleProject, toggleProjects]);
}
