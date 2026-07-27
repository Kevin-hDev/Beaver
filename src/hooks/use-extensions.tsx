import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { useFsEvent } from "@/hooks/use-fs-event";
import { ExtensionActivationDialog } from "@/components/extensions/extension-activation-dialog";
import type { ExtensionHostStatus, ExtensionRecord } from "@/types/extensions";

const EMPTY_HOST: ExtensionHostStatus = {
  state: "stopped",
  jitiVersion: "",
  apiVersion: "1",
  activeExtensions: 0,
  diagnostics: [],
};

function useExtensionsState() {
  const [extensions, setExtensions] = useState<ExtensionRecord[]>([]);
  const [host, setHost] = useState<ExtensionHostStatus>(EMPTY_HOST);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const [operationError, setOperationError] = useState(false);
  const [busyIds, setBusyIds] = useState<Set<string>>(() => new Set());
  const [pendingActivation, setPendingActivation] =
    useState<ExtensionRecord | null>(null);

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

  const applyEnabled = useCallback((
    id: string,
    enabled: boolean,
    trustConfirmed = false,
  ) =>
    mutate(id, "set_extension_enabled", {
      extensionId: id,
      enabled,
      trustConfirmed,
    },
      (record) => ({ ...record, enabled })), [mutate]);

  const setEnabled = useCallback((id: string, enabled: boolean) => {
    const extension = extensions.find((record) => record.manifest.id === id);
    if (enabled && extension && extension.kind !== "builtin" && !extension.trusted) {
      setPendingActivation(extension);
      return Promise.resolve();
    }
    return applyEnabled(id, enabled);
  }, [applyEnabled, extensions]);

  const confirmActivation = useCallback(async () => {
    if (!pendingActivation) return;
    const id = pendingActivation.manifest.id;
    await applyEnabled(id, true, true);
    setPendingActivation(null);
  }, [applyEnabled, pendingActivation]);

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
    pendingActivation,
    refresh,
    addLocal,
    setEnabled,
    confirmActivation,
    cancelActivation: () => setPendingActivation(null),
    setShowInChat,
    remove: (id: string) => run("remove_extension", { extensionId: id }),
    reload: () => run("reload_extension_host"),
    recover: () => run("recover_without_user_extensions"),
    openSource: (id: string) => run("open_extension_source", { extensionId: id }),
  };
}

type ExtensionsContextValue = ReturnType<typeof useExtensionsState>;
const ExtensionsContext = createContext<ExtensionsContextValue | null>(null);

export function ExtensionsProvider({ children }: { children: ReactNode }) {
  const value = useExtensionsState();
  return (
    <ExtensionsContext.Provider value={value}>
      {children}
      {value.pendingActivation && (
        <ExtensionActivationDialog
          extension={value.pendingActivation}
          busy={value.busyIds.has(value.pendingActivation.manifest.id)}
          onCancel={value.cancelActivation}
          onConfirm={() => void value.confirmActivation()}
        />
      )}
    </ExtensionsContext.Provider>
  );
}

export function useExtensions() {
  const context = useContext(ExtensionsContext);
  if (!context) throw new Error("ExtensionsProvider is missing");
  return context;
}
