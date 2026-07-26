import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import type { Project } from "@/types/agent";
import { AGENT_SESSIONS_CHANGED } from "./agent-session-events";

export function useProjects() {
  const [projects, setProjects] = useState<Project[]>([]);

  const refresh = useCallback(async () => {
    const list = await invoke<Project[]>("list_projects");
    setProjects(list.sort((a, b) => a.order - b.order));
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
      } catch {
        showToast(i18n.t("errors.projectRenameFailed"), "error");
      }
    },
    [refresh],
  );

  const remove = useCallback(
    async (id: string) => {
      try {
        await invoke("delete_project", { id });
        await refresh();
      } catch {
        showToast(i18n.t("errors.projectDeleteFailed"), "error");
      }
    },
    [refresh],
  );

  const reorder = useCallback(
    async (ids: string[]) => {
      try {
        await invoke("reorder_projects", { ids });
        await refresh();
      } catch {
        showToast(i18n.t("errors.saveFailed"), "error");
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

  return { projects, refresh, add, rename, remove, reorder, openFolder };
}
