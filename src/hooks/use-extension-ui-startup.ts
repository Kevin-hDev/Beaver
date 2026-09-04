import { invoke } from "@tauri-apps/api/core";
import { createContext, useCallback, useContext, useMemo, useState } from "react";
import {
  parseExtensionUiStartupState,
} from "@/lib/extension-ui-startup";
import type {
  ExtensionUiIncident,
  ExtensionUiStartupState,
} from "@/types/extensions";

export interface ExtensionUiStartupController {
  state: ExtensionUiStartupState;
  incident: ExtensionUiIncident | null;
  busy: boolean;
  error: boolean;
  continueSafe: () => Promise<boolean>;
  retry: () => Promise<boolean>;
  discardInvalid: () => Promise<boolean>;
  discardInterrupted: (extensionId: string) => Promise<boolean>;
  resolveIncident: (extensionId: string) => void;
  refresh: () => Promise<boolean>;
}

export const ExtensionUiStartupContext = createContext<ExtensionUiStartupController | null>(null);

export function useExtensionUiStartup(
  initial: ExtensionUiStartupState,
): ExtensionUiStartupController {
  const [state, setState] = useState(initial);
  const [incident, setIncident] = useState<ExtensionUiIncident | null>(() => incidentFrom(initial));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(false);
  const run = useCallback(async (
    command: string,
    payload: Record<string, unknown> = {},
  ): Promise<boolean> => {
    if (busy) return false;
    setBusy(true);
    setError(false);
    try {
      const next = parseExtensionUiStartupState(await invoke<unknown>(command, payload));
      setState(next);
      if (next.mode.kind === "pendingInterruptedUi") setIncident(next.mode);
      else if (next.mode.kind === "normal") setIncident(null);
      return true;
    } catch {
      setError(true);
      return false;
    } finally {
      setBusy(false);
    }
  }, [busy]);
  return useMemo(() => ({
    state,
    incident,
    busy,
    error,
    continueSafe: () => run("continue_without_extension_ui"),
    retry: () => run("retry_interrupted_extension_ui"),
    discardInvalid: () => run("discard_invalid_extension_ui_marker"),
    discardInterrupted: async (extensionId: string) => {
      const succeeded = await run("discard_interrupted_extension_ui_marker", { extensionId });
      if (succeeded) {
        setIncident((current) => current?.extensionId === extensionId ? null : current);
      }
      return succeeded;
    },
    // The registry mutation already cleared the durable marker; only its stale
    // in-memory projection remains to be removed here.
    resolveIncident: (extensionId: string) => {
      setIncident((current) => current?.extensionId === extensionId ? null : current);
    },
    refresh: () => run("get_extension_ui_startup_state"),
  }), [busy, error, incident, run, state]);
}

export function useExtensionUiStartupContext(): ExtensionUiStartupController | null {
  return useContext(ExtensionUiStartupContext);
}

function incidentFrom(state: ExtensionUiStartupState): ExtensionUiIncident | null {
  return state.mode.kind === "pendingInterruptedUi" ? state.mode : null;
}
