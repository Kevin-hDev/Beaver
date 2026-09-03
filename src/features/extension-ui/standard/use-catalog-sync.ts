import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { parseStandardCatalog } from "./catalog-parser";
import type { StandardCatalogSnapshot } from "./types";

export type CatalogSyncState =
  | { kind: "loading"; snapshot: null }
  | { kind: "empty"; snapshot: StandardCatalogSnapshot }
  | { kind: "ready"; snapshot: StandardCatalogSnapshot }
  | { kind: "error"; snapshot: null }
  | { kind: "stale-error"; snapshot: StandardCatalogSnapshot };

export function useCatalogSync(): CatalogSyncState {
  const [state, setState] = useState<CatalogSyncState>({ kind: "loading", snapshot: null });

  useEffect(() => {
    let live: StandardCatalogSnapshot | null = null;
    let requestedRevision = 0;
    let reconciling = false;
    let pending = false;
    let disposed = false;

    const reconcile = async () => {
      if (disposed) return;
      if (reconciling) {
        pending = true;
        return;
      }
      reconciling = true;
      try {
        let passes = 0;
        do {
          pending = false;
          passes += 1;
          const target = requestedRevision;
          try {
            const parsed = parseStandardCatalog(await invoke<unknown>("get_extension_ui_catalog"));
            if (disposed) return;
            if (!live || parsed.revision >= live.revision) {
              live = parsed;
              setState({
                kind: parsed.contributions.length === 0 ? "empty" : "ready",
                snapshot: parsed,
              });
            }
          } catch {
            if (disposed) return;
            setState(live
              ? { kind: "stale-error", snapshot: live }
              : { kind: "error", snapshot: null });
          }
          if (!pending && (requestedRevision <= target
            || (live && live.revision >= requestedRevision))) break;
        } while (!disposed && passes < 2);
      } finally {
        reconciling = false;
        if (pending && !disposed) queueMicrotask(() => void reconcile());
      }
    };

    void reconcile();
    const unlisten = listen<number>("extensions-ui-catalog-changed", ({ payload }) => {
      if (!Number.isSafeInteger(payload) || payload < 0) return;
      requestedRevision = Math.max(requestedRevision, payload);
      void reconcile();
    });
    return () => {
      disposed = true;
      pending = false;
      cleanupTauriListener(unlisten);
    };
  }, []);

  return state;
}
