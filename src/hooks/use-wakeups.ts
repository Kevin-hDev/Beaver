import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AutomationErrorCode,
  AutomationMigrationStatus,
  CreateWakeupInput,
  HeartbeatConfig,
  ScheduledWakeup,
  UpdateWakeupInput,
  WakeupDetail,
  WakeupHistoryPage,
} from "@/types/wakeup";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { useFsEvent } from "./use-fs-event";

const ERROR_CODES = new Set<AutomationErrorCode>([
  "audit_unavailable", "store_unavailable", "migration_unavailable", "invalid_timezone",
  "invalid_schedule", "model_unavailable", "provider_unavailable", "not_found",
  "globally_paused", "invalid_project", "invalid_input",
]);

function errorCode(error: unknown): AutomationErrorCode {
  return typeof error === "string" && ERROR_CODES.has(error as AutomationErrorCode)
    ? error as AutomationErrorCode
    : "store_unavailable";
}

export function useWakeups() {
  const [wakeups, setWakeups] = useState<ScheduledWakeup[]>([]);
  const [globalPaused, setGlobalPaused] = useState(false);
  const [migration, setMigration] = useState<AutomationMigrationStatus | null>(null);
  const [detail, setDetail] = useState<WakeupDetail | null>(null);
  const [history, setHistory] = useState<WakeupHistoryPage>({ entries: [], next_cursor: null });
  const [loading, setLoading] = useState(true);
  const [detailLoading, setDetailLoading] = useState(false);
  const [error, setError] = useState<AutomationErrorCode | null>(null);
  const detailRequest = useRef(0);
  const reportError = useCallback((cause: unknown) => {
    setError(errorCode(cause));
  }, []);

  const refresh = useCallback(async () => {
    try {
      const [list, heartbeat, migrationStatus] = await Promise.all([
        invoke<ScheduledWakeup[]>("list_wakeups"),
        invoke<HeartbeatConfig>("get_heartbeat_config"),
        invoke<AutomationMigrationStatus>("reconcile_automation_migration", { timezone: null }),
      ]);
      setWakeups(list);
      setGlobalPaused(heartbeat.global_paused);
      setMigration(migrationStatus);
      setError(null);
    } catch (cause) {
      setError(errorCode(cause));
    } finally {
      setLoading(false);
    }
  }, []);

  const loadDetail = useCallback(async (id: string) => {
    const request = ++detailRequest.current;
    setDetailLoading(true);
    try {
      const [nextDetail, nextHistory] = await Promise.all([
        invoke<WakeupDetail>("get_wakeup", { automationId: id }),
        invoke<WakeupHistoryPage>("list_wakeup_runs", { wakeupId: id, limit: 20, cursor: null }),
      ]);
      if (request !== detailRequest.current) return;
      setDetail(nextDetail);
      setHistory(nextHistory);
      setError(null);
    } catch (cause) {
      if (request === detailRequest.current) setError(errorCode(cause));
    } finally {
      if (request === detailRequest.current) setDetailLoading(false);
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
  }, [refresh]);

  useFsEvent("fs:config-changed", () => void refresh());
  useFsEvent("fs:logs-changed", () => void refresh());

  useEffect(() => {
    const refreshFromEvent = () => void refresh();
    const completed = listen("wakeup-completed", refreshFromEvent);
    const failed = listen("wakeup-failed", refreshFromEvent);
    return () => {
      cleanupTauriListener(completed);
      cleanupTauriListener(failed);
    };
  }, [refresh]);

  const create = useCallback(async (input: CreateWakeupInput) => {
    try {
      const created = await invoke<WakeupDetail>("create_wakeup", { input });
      await refresh();
      return created;
    } catch (cause) {
      setError(errorCode(cause));
      throw cause;
    }
  }, [refresh]);

  const update = useCallback(async (input: UpdateWakeupInput) => {
    try {
      const updated = await invoke<WakeupDetail>("update_wakeup", { input });
      await refresh();
      await loadDetail(input.automation_id);
      return updated;
    } catch (cause) {
      setError(errorCode(cause));
      throw cause;
    }
  }, [loadDetail, refresh]);

  const remove = useCallback(async (id: string) => {
    try {
      await invoke("delete_wakeup", { id });
      setDetail(null);
      setHistory({ entries: [], next_cursor: null });
      await refresh();
    } catch (cause) { reportError(cause); }
  }, [refresh, reportError]);

  const toggle = useCallback(async (id: string, active: boolean) => {
    try {
      await invoke("set_wakeup_active", { id, active });
      await refresh();
    } catch (cause) { reportError(cause); }
  }, [refresh, reportError]);

  const setPaused = useCallback(async (paused: boolean) => {
    try {
      await invoke("set_global_paused", { paused });
      await refresh();
    } catch (cause) { reportError(cause); }
  }, [refresh, reportError]);

  const loadMoreHistory = useCallback(async () => {
    const id = detail?.definition.id;
    if (!id || !history.next_cursor) return;
    try {
      const page = await invoke<WakeupHistoryPage>("list_wakeup_runs", {
        wakeupId: id,
        limit: 20,
        cursor: history.next_cursor,
      });
      setHistory((current) => ({
        entries: [...current.entries, ...page.entries],
        next_cursor: page.next_cursor,
      }));
    } catch (cause) { reportError(cause); }
  }, [detail?.definition.id, history.next_cursor, reportError]);

  const chooseTimezone = useCallback(async (timezone: string) => {
    try {
      const status = await invoke<AutomationMigrationStatus>("reconcile_automation_migration", { timezone });
      setMigration(status);
      await refresh();
    } catch (cause) { reportError(cause); }
  }, [refresh, reportError]);

  const resolveConflict = useCallback(async (
    legacyId: string,
    decision: "remove_historical" | "import_as_new",
    timezone?: string,
  ) => {
    try {
      await invoke("resolve_automation_migration_conflict", { legacyId, decision, timezone });
      await refresh();
    } catch (cause) { reportError(cause); }
  }, [refresh, reportError]);

  return {
    wakeups, globalPaused, migration, detail, history, loading, detailLoading, error,
    refresh, loadDetail, loadMoreHistory, create, update, remove, toggle, setPaused,
    chooseTimezone, resolveConflict,
  };
}
