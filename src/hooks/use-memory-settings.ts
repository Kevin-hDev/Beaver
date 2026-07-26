import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  MemoryMode,
  MemoryOverview,
  MemoryScopeOverview,
} from "./memory-settings-types";

export type {
  MemoryMode,
  MemoryOverview,
  MemoryScopeOverview,
  MemoryTopic,
} from "./memory-settings-types";

export function useMemorySettings(sessionId?: string | null) {
  const [overview, setOverview] = useState<MemoryOverview | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [loadingProjectId, setLoadingProjectId] = useState<string | null>(null);
  const loadSequence = useRef(0);
  const modeSequence = useRef(0);
  const budgetSequence = useRef(0);
  const archiveSequence = useRef(0);
  const projectSequence = useRef(0);
  const loadingProject = useRef<string | null>(null);

  const refresh = useCallback(() => {
    const sequence = loadSequence.current + 1;
    loadSequence.current = sequence;
    projectSequence.current += 1;
    loadingProject.current = null;
    setLoadingProjectId(null);
    setLoading(true);
    invoke<MemoryOverview>("get_memory_overview", {
      workingDir: null,
      sessionId: sessionId ?? null,
    })
      .then((result) => {
        if (loadSequence.current !== sequence) return;
        setOverview(result);
        setError(false);
      })
      .catch(() => {
        if (loadSequence.current === sequence) setError(true);
      })
      .finally(() => {
        if (loadSequence.current === sequence) setLoading(false);
      });
  }, [sessionId]);

  useEffect(() => {
    const sequence = loadSequence.current + 1;
    loadSequence.current = sequence;
    projectSequence.current += 1;
    loadingProject.current = null;
    invoke<MemoryOverview>("get_memory_overview", {
      workingDir: null,
      sessionId: sessionId ?? null,
    })
      .then((result) => {
        if (loadSequence.current !== sequence) return;
        setOverview(result);
        setError(false);
      })
      .catch(() => {
        if (loadSequence.current === sequence) setError(true);
      })
      .finally(() => {
        if (loadSequence.current === sequence) setLoading(false);
      });
    return () => {
      loadSequence.current += 1;
      projectSequence.current += 1;
    };
  }, [sessionId]);

  const setMode = useCallback((mode: MemoryMode) => {
    if (!overview) return;
    const sequence = modeSequence.current + 1;
    modeSequence.current = sequence;
    const previous = overview.settings.mode;
    setOverview((current) => current && ({
      ...current,
      settings: { ...current.settings, mode },
    }));
    invoke<MemoryOverview["settings"]>("set_memory_mode", { mode })
      .then((settings) => {
        if (modeSequence.current === sequence) {
          setOverview((current) => current && ({
            ...current,
            settings: { ...current.settings, mode: settings.mode },
          }));
          setError(false);
        }
      })
      .catch(() => {
        if (modeSequence.current === sequence) {
          setOverview((current) => current && ({
            ...current,
            settings: { ...current.settings, mode: previous },
          }));
          setError(true);
        }
      });
  }, [overview]);

  const setBudget = useCallback((tokens: number) => {
    if (!overview) return;
    const sequence = budgetSequence.current + 1;
    budgetSequence.current = sequence;
    const previous = overview.settings.contextBudgetTokens;
    setOverview((current) => current && ({
      ...current,
      settings: { ...current.settings, contextBudgetTokens: tokens },
    }));
    invoke<MemoryOverview["settings"]>("set_memory_context_budget", { tokens })
      .then((settings) => {
        if (budgetSequence.current === sequence) {
          setOverview((current) => current && ({
            ...current,
            settings: {
              ...current.settings,
              contextBudgetTokens: settings.contextBudgetTokens,
            },
          }));
          setError(false);
        }
      })
      .catch(() => {
        if (budgetSequence.current === sequence) {
          setOverview((current) => current && ({
            ...current,
            settings: { ...current.settings, contextBudgetTokens: previous },
          }));
          setError(true);
        }
      });
  }, [overview]);

  const archiveTopic = useCallback((path: string) => {
    const sequence = archiveSequence.current + 1;
    archiveSequence.current = sequence;
    invoke<MemoryOverview>("archive_memory_topic", {
      path,
      sessionId: sessionId ?? null,
    })
      .then((result) => {
        if (archiveSequence.current === sequence) {
          setOverview(result);
          setError(false);
        }
      })
      .catch(() => {
        if (archiveSequence.current === sequence) setError(true);
      });
  }, [sessionId]);

  const loadProjectTopics = useCallback((projectId: string) => {
    if (loadingProject.current) return;
    const sequence = projectSequence.current + 1;
    projectSequence.current = sequence;
    loadingProject.current = projectId;
    setLoadingProjectId(projectId);
    invoke<MemoryScopeOverview>("get_memory_project_topics", { projectId })
      .then((scope) => {
        if (projectSequence.current !== sequence) return;
        setOverview((current) => current && ({
          ...current,
          otherProjects: current.otherProjects.map((item) => (
            item.id === projectId ? scope : item
          )),
        }));
        setError(false);
      })
      .catch(() => {
        if (projectSequence.current === sequence) setError(true);
      })
      .finally(() => {
        if (projectSequence.current === sequence) {
          loadingProject.current = null;
          setLoadingProjectId(null);
        }
      });
  }, []);

  return {
    overview,
    loading,
    error,
    loadingProjectId,
    refresh,
    setMode,
    setBudget,
    archiveTopic,
    loadProjectTopics,
  };
}
