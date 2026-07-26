import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CreateWakeupInput,
  HeartbeatConfig,
  WakeupRun,
  WakeupStatusSummary,
  ScheduledWakeup,
} from "@/types/wakeup";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { useFsEvent } from "./use-fs-event";

export function useWakeups() {
  const [wakeups, setWakeups] = useState<ScheduledWakeup[]>([]);
  const [runs, setRuns] = useState<WakeupRun[]>([]);
  const [summaries, setSummaries] = useState<Record<string, WakeupStatusSummary>>({});
  const [globalPaused, setGlobalPaused] = useState(false);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const [list, hb, statusList, runList] = await Promise.all([
        invoke<ScheduledWakeup[]>("list_wakeups"),
        invoke<HeartbeatConfig>("get_heartbeat_config"),
        invoke<WakeupStatusSummary[]>("get_wakeup_status_summaries"),
        invoke<WakeupRun[]>("list_wakeup_runs", { wakeupId: null }),
      ]);
      setWakeups(list);
      setGlobalPaused(hb.global_paused);
      setRuns(runList);
      setSummaries(Object.fromEntries(statusList.map((item) => [item.wakeup_id, item])));
    } catch {
      setWakeups([]);
      setRuns([]);
      setSummaries({});
      setGlobalPaused(false);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
  }, [refresh]);

  useFsEvent("fs:config-changed", () => void refresh());
  useFsEvent("fs:logs-changed", () => void refresh());

  useEffect(() => {
    const refreshFromEvent = () => {
      void refresh();
    };
    const unlistenCompleted = listen("wakeup-completed", refreshFromEvent);
    const unlistenFailed = listen("wakeup-failed", refreshFromEvent);
    return () => {
      cleanupTauriListener(unlistenCompleted);
      cleanupTauriListener(unlistenFailed);
    };
  }, [refresh]);

  const create = useCallback(
    async (input: CreateWakeupInput) => {
      const w = await invoke<ScheduledWakeup>("create_wakeup", { input });
      await refresh();
      return w;
    },
    [refresh],
  );

  const update = useCallback(
    async (wakeup: ScheduledWakeup) => {
      await invoke("update_wakeup", { wakeup });
      await refresh();
    },
    [refresh],
  );

  const remove = useCallback(
    async (id: string) => {
      await invoke("delete_wakeup", { id });
      await refresh();
    },
    [refresh],
  );

  const toggle = useCallback(
    async (id: string, active: boolean) => {
      await invoke("set_wakeup_active", { id, active });
      await refresh();
    },
    [refresh],
  );

  const setPaused = useCallback(
    async (paused: boolean) => {
      await invoke("set_global_paused", { paused });
      await refresh();
    },
    [refresh],
  );

  return {
    wakeups,
    runs,
    summaries,
    globalPaused,
    loading,
    refresh,
    create,
    update,
    remove,
    toggle,
    setPaused,
  };
}
