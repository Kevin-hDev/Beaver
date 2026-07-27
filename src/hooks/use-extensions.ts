import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useFsEvent } from "@/hooks/use-fs-event";
import type { ExtensionHostStatus, ExtensionRecord } from "@/types/extensions";

const EMPTY_HOST: ExtensionHostStatus = {
  state: "stopped",
  jitiVersion: "2.7.0",
  apiVersion: "1",
  activeExtensions: 0,
};

export function useExtensions() {
  const [extensions, setExtensions] = useState<ExtensionRecord[]>([]);
  const [host, setHost] = useState<ExtensionHostStatus>(EMPTY_HOST);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const [operationError, setOperationError] = useState(false);
  const [busyIds, setBusyIds] = useState<Set<string>>(() => new Set());

  const refresh = useCallback(async () => {
    try {
      const [records, status] = await Promise.all([
        invoke<ExtensionRecord[]>("list_extensions"),
        invoke<ExtensionHostStatus>("get_extension_host_status"),
      ]);
      setExtensions(records);
      setHost(status);
      setLoadError(false);
    } catch {
      setLoadError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- refresh synchronise le registre possédé par Rust
    void refresh();
  }, [refresh]);

  const refreshFromEvent = useCallback(() => { void refresh(); }, [refresh]);
  useFsEvent("fs:extensions-changed", refreshFromEvent);

  const mutate = useCallback(async (
    id: string,
    command: string,
    payload: Record<string, unknown>,
    optimistic: (record: ExtensionRecord) => ExtensionRecord,
  ) => {
    setOperationError(false);
    setBusyIds((current) => new Set([...current, id]));
    setExtensions((current) =>
      current.map((record) => record.manifest.id === id ? optimistic(record) : record));
    try {
      await invoke(command, payload);
    } catch {
      setOperationError(true);
    } finally {
      await refresh();
      setBusyIds((current) => {
        const next = new Set(current);
        next.delete(id);
        return next;
      });
    }
  }, [refresh]);

  const setEnabled = useCallback((id: string, enabled: boolean) =>
    mutate(id, "set_extension_enabled", { extensionId: id, enabled },
      (record) => ({ ...record, enabled })), [mutate]);

  const setShowInChat = useCallback((id: string, showInChat: boolean) =>
    mutate(id, "set_extension_show_in_chat", { extensionId: id, showInChat },
      (record) => ({ ...record, showInChat })), [mutate]);

  const addLocal = useCallback(async (path: string) => {
    setOperationError(false);
    try {
      const added = await invoke<ExtensionRecord>("add_local_extension", { path });
      await refresh();
      return added;
    } catch {
      setOperationError(true);
      return null;
    }
  }, [refresh]);

  const run = useCallback(async (command: string, payload: Record<string, unknown> = {}) => {
    setOperationError(false);
    try {
      await invoke(command, payload);
    } catch {
      setOperationError(true);
    } finally {
      await refresh();
    }
  }, [refresh]);

  return {
    extensions,
    host,
    loading,
    loadError,
    operationError,
    busyIds,
    refresh,
    addLocal,
    setEnabled,
    setShowInChat,
    remove: (id: string) => run("remove_extension", { extensionId: id }),
    reload: () => run("reload_extension_host"),
    recover: () => run("recover_without_user_extensions"),
    openSource: (id: string) => run("open_extension_source", { extensionId: id }),
  };
}
