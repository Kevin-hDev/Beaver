import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useFsEvent } from "@/hooks/use-fs-event";

export type PermissionMode = "auto" | "manual" | "chat";
export type PermissionFamily = "chat" | "tools";

interface AgentSettings {
  permission_mode: PermissionMode;
}

interface SessionPermissionState {
  permission_family?: PermissionFamily | null;
  permission_mode: PermissionMode;
}

const ALL_MODES: PermissionMode[] = ["chat", "manual", "auto"];
let defaultMode: PermissionMode = "auto";

export function usePermissionMode(sessionId?: string) {
  const [mode, setMode] = useState<PermissionMode>(defaultMode);
  const [family, setFamily] = useState<PermissionFamily | null>(null);
  const [loaded, setLoaded] = useState(false);
  const reload = useCallback(async () => {
    try {
      if (sessionId) {
        const state = await invoke<SessionPermissionState>("get_session_permission_state", { id: sessionId });
        setMode(state.permission_mode);
        setFamily(state.permission_family ?? null);
      } else {
        const settings = await invoke<AgentSettings>("get_agent_settings");
        defaultMode = settings.permission_mode;
        setMode(defaultMode);
        setFamily(null);
      }
    } catch {
      if (!sessionId) setMode(defaultMode);
    } finally {
      setLoaded(true);
    }
  }, [sessionId]);

  useEffect(() => {
    queueMicrotask(() => void reload());
  }, [reload]);

  const reloadFromEvent = useCallback(() => {
    void reload();
  }, [reload]);
  useFsEvent("fs:config-changed", reloadFromEvent);

  const availableModes = useMemo<PermissionMode[]>(() => {
    if (family === "chat") return ["chat"];
    if (family === "tools") return ["manual", "auto"];
    return ALL_MODES;
  }, [family]);

  const change = useCallback(async (next: PermissionMode) => {
    if (sessionId) {
      try {
        const state = await invoke<SessionPermissionState>("set_session_permission_mode", {
          id: sessionId,
          mode: next,
        });
        setMode(state.permission_mode ?? next);
        setFamily(state.permission_family ?? null);
      } catch {
        await reload();
        return;
      }
    } else {
      setMode(next);
    }
    defaultMode = next;
    try {
      await invoke("set_permission_mode", { mode: next });
    } catch {
      // Le mode de session reste autoritaire même si le défaut global échoue.
    }
  }, [reload, sessionId]);

  const toggle = useCallback(() => {
    const idx = availableModes.indexOf(mode);
    void change(availableModes[(idx + 1) % availableModes.length]);
  }, [availableModes, mode, change]);

  useEffect(() => {
    if (!loaded) return;
    const onKey = (event: KeyboardEvent) => {
      if (!event.shiftKey || !event.key.startsWith("Tab")) return;
      const target = event.target as HTMLElement | null;
      const tag = target?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || target?.isContentEditable) return;
      event.preventDefault();
      toggle();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [loaded, toggle]);

  return { mode, family, availableModes, change, toggle, refresh: reload, loaded };
}
