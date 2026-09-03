import { invoke } from "@tauri-apps/api/core";
import { createContext, useCallback, useContext, useMemo, useState } from "react";
import {
  parseExtensionUiStartupState,
} from "@/lib/extension-ui-startup";
import type { ExtensionUiStartupState } from "@/types/extensions";

export interface ExtensionUiStartupController {
  state: ExtensionUiStartupState;
  busy: boolean;
  error: boolean;
  continueSafe: () => Promise<boolean>;
  retry: () => Promise<boolean>;
  discardInvalid: () => Promise<boolean>;
}

export const ExtensionUiStartupContext = createContext<ExtensionUiStartupController | null>(null);

export function useExtensionUiStartup(
  initial: ExtensionUiStartupState,
): ExtensionUiStartupController {
  const [state, setState] = useState(initial);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(false);
  const run = useCallback(async (command: string): Promise<boolean> => {
    if (busy) return false;
    setBusy(true);
    setError(false);
    try {
      setState(parseExtensionUiStartupState(await invoke<unknown>(command)));
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
    busy,
    error,
    continueSafe: () => run("continue_without_extension_ui"),
    retry: () => run("retry_interrupted_extension_ui"),
    discardInvalid: () => run("discard_invalid_extension_ui_marker"),
  }), [busy, error, run, state]);
}

export function useExtensionUiStartupContext(): ExtensionUiStartupController | null {
  return useContext(ExtensionUiStartupContext);
}
