import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import { localStoreErrorMessage } from "@/lib/local-store-error";
import i18n from "@/i18n";
import type { Project } from "@/types/agent";
import { AGENT_SESSIONS_CHANGED } from "./agent-session-events";

export type ProjectLoadState = "loading" | "ready" | "error";

export function useProjects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loadState, setLoadState] = useState<ProjectLoadState>("loading");

  const refresh = useCallback(async () => {
    setLoadState("loading");
    try {
      const list = await invoke<Project[]>("list_projects");
      setProjects(list.sort((a, b) => a.order - b.order));
      setLoadState("ready");
    } catch (error) {
      setLoadState("error");
      showToast(localStoreErrorMessage(error, i18n.t), "error");
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
    const refreshProjects = () => void refresh();
    window.addEventListener(AGENT_SESSIONS_CHANGED, refreshProjects);
    return () => window.removeEventListener(AGENT_SESSIONS_CHANGED, refreshProjects);
  }, [refresh]);

  const add = useCallback(
    async (path: string): Promise<Project> => {
      const project = await invoke<Project>("add_project", { path });
      await refresh();
      return project;
    },
    [refresh],
  );

  const rename = useCallback(
    async (id: string, name: string) => {
      try {
        await invoke("rename_project", { id, name });
        await refresh();
      } catch (error) {
        showToast(localStoreErrorMessage(error, i18n.t), "error");
      }
    },
    [refresh],
  );

  const remove = useCallback(
    async (id: string) => {
      try {
        await invoke("delete_project", { id });
        await refresh();
      } catch (error) {
        showToast(localStoreErrorMessage(error, i18n.t), "error");
      }
    },
    [refresh],
  );

  const reorder = useCallback(
    async (ids: string[]) => {
      try {
        await invoke("reorder_projects", { ids });
        await refresh();
      } catch (error) {
        showToast(localStoreErrorMessage(error, i18n.t), "error");
      }
    },
    [refresh],
  );

  const openFolder = useCallback(async (path: string) => {
    try {
      await invoke("open_project_folder", { path });
    } catch {
      showToast(i18n.t("errors.projectOpenFailed"), "error");
    }
  }, []);

  return {
    projects,
    loadState,
    refresh,
    add,
    rename,
    remove,
    reorder,
    openFolder,
  };
}
