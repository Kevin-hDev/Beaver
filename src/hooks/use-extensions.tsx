import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { useFsEvent } from "@/hooks/use-fs-event";
import { useExtensionInstall } from "@/hooks/use-extension-install";
import { useExtensionPriorities } from "@/hooks/use-extension-priorities";
import { useExtensionRecovery } from "@/hooks/use-extension-recovery";
import { ExtensionActivationDialog } from "@/components/extensions/extension-activation-dialog";
import { extensionErrorKey } from "@/lib/extension-errors";
import {
  EMPTY_EXTENSION_HOST,
  parseExtensionHostStatus,
} from "@/lib/extension-host-status";
import { parseExtensionRecords } from "@/lib/extension-records";
import type { ExtensionHostStatus, ExtensionRecord } from "@/types/extensions";

function useExtensionsState() {
  const [extensions, setExtensions] = useState<ExtensionRecord[]>([]);
  const [host, setHost] = useState<ExtensionHostStatus>(EMPTY_EXTENSION_HOST);
  const [hostLoaded, setHostLoaded] = useState(false);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [operationError, setOperationError] = useState<string | null>(null);
  const [hostOperations, setHostOperations] = useState(0);
  const refreshGeneration = useRef(0);
  const {
    protectedPluginIds,
    priorityBusy,
    applyValue: applyPriorityValue,
    setPriorityPlugins,
  } = useExtensionPriorities(setOperationError);
  const [busyIds, setBusyIds] = useState<Set<string>>(() => new Set());
  const [pendingActivation, setPendingActivation] =
    useState<ExtensionRecord | null>(null);

  const refresh = useCallback(async () => {
    const generation = ++refreshGeneration.current;
    setLoading(true);
    setLoadError(null);
    try {
      const [records, status, preferences] = await Promise.all([
        invoke<unknown>("list_extensions"),
        invoke<unknown>("get_extension_host_status"),
        invoke<unknown>("get_extension_discovery_preferences"),
      ]);
      if (generation !== refreshGeneration.current) return;
      setExtensions(parseExtensionRecords(records));
      setHost(parseExtensionHostStatus(status));
      setHostLoaded(true);
      applyPriorityValue(preferences);
      setLoadError(null);
    } catch (error) {
      if (generation !== refreshGeneration.current) return;
      setLoadError(extensionErrorKey(error, "extensions.errors.load"));
    } finally {
      if (generation === refreshGeneration.current) setLoading(false);
    }
  }, [applyPriorityValue]);

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
    const ownsHost = command === "set_extension_enabled";
    setOperationError(null);
    if (ownsHost) setHostOperations((count) => count + 1);
    setBusyIds((current) => new Set([...current, id]));
    setExtensions((current) =>
      current.map((record) => record.manifest.id === id ? optimistic(record) : record));
    try {
      const reminder = await invoke<boolean | void>(command, payload);
      if (reminder === true) {
        setOperationError("extensions.sensitiveAccessReminder");
      }
      return true;
    } catch (error) {
      setOperationError(extensionErrorKey(error, "extensions.errors.operation"));
      return false;
    } finally {
      await refresh();
      if (ownsHost) setHostOperations((count) => Math.max(0, count - 1));
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
      setOperationError(null);
      setPendingActivation(extension);
      return Promise.resolve();
    }
    return applyEnabled(id, enabled).then(() => undefined);
  }, [applyEnabled, extensions]);

  const confirmActivation = useCallback(async () => {
    if (!pendingActivation) return;
    const id = pendingActivation.manifest.id;
    const succeeded = await applyEnabled(id, true, true);
    if (succeeded) setPendingActivation(null);
  }, [applyEnabled, pendingActivation]);

  const setShowInChat = useCallback((id: string, showInChat: boolean) =>
    mutate(id, "set_extension_show_in_chat", { extensionId: id, showInChat },
      (record) => ({ ...record, showInChat })), [mutate]);

  const install = useExtensionInstall(refresh, setOperationError);

  const run = useCallback(async (command: string, payload: Record<string, unknown> = {}) => {
    setOperationError(null);
    try {
      const reminder = await invoke<boolean | void>(command, payload);
      if (reminder === true) {
        setOperationError("extensions.sensitiveAccessReminder");
      }
    } catch (error) {
      setOperationError(extensionErrorKey(error, "extensions.errors.operation"));
    } finally {
      await refresh();
    }
  }, [refresh]);

  const runHost = useCallback(async (operation: () => Promise<void>) => {
    setHostOperations((count) => count + 1);
    try {
      await operation();
    } finally {
      setHostOperations((count) => Math.max(0, count - 1));
    }
  }, []);
  const recovery = useExtensionRecovery(run, runHost, setOperationError);

  return {
    extensions,
    host,
    hostLoaded,
    loading,
    loadError,
    operationError,
    hostBusy: hostOperations > 0 || recovery.recoveryBusy,
    busyIds,
    protectedPluginIds,
    priorityBusy,
    pendingActivation,
    refresh,
    addLocal: (path: string) => install("add_local_extension", { path }),
    installGit: (url: string) => install("install_git_extension", { url }),
    installNpm: (packageSpec: string) =>
      install("install_npm_extension", { packageSpec }),
    setEnabled,
    confirmActivation,
    cancelActivation: () => setPendingActivation(null),
    setShowInChat,
    setPriorityPlugins,
    update: (id: string) => mutate(
      id,
      "update_extension",
      { extensionId: id },
      (record) => record,
    ),
    remove: (id: string) => run("remove_extension", { extensionId: id }),
    reload: () => runHost(() => run("reload_extension_host")),
    recover: () => runHost(() => run("recover_extension_host")),
    openSource: (id: string) => run("open_extension_source", { extensionId: id }),
    ...recovery,
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
          errorKey={value.operationError}
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
