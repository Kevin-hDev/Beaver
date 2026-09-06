import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { parseInstallJobsSnapshot } from "@/lib/extension-install-job-parser";
import { installJobErrorKey } from "@/lib/extension-install-job-errors";
import type { InstallJobsSnapshot } from "@/types/extension-install-jobs.generated";

const EMPTY_SNAPSHOT: InstallJobsSnapshot = { revision: 0, jobs: [] };

export function useExtensionInstallSnapshots() {
  const [loading, setLoading] = useState(true);
  const [snapshot, setSnapshot] = useState(EMPTY_SNAPSHOT);
  const [loadError, setLoadError] = useState<string | null>(null);
  const revision = useRef(-1);
  const mounted = useRef(false);
  const accept = useCallback((input: unknown) => {
    const next = parseInstallJobsSnapshot(input);
    if (!mounted.current) return;
    if (next.revision > revision.current) {
      revision.current = next.revision;
      setSnapshot(next);
    }
    setLoadError(null);
    setLoading(false);
  }, []);
  const reportError = useCallback((error: unknown) => {
    if (mounted.current) {
      setLoadError(installJobErrorKey(error, "extensionInstalls.errors.load"));
      setLoading(false);
    }
  }, []);
  const refresh = useCallback(async () => {
    try { accept(await invoke<unknown>("list_extension_installs")); }
    catch (error) { reportError(error); }
  }, [accept, reportError]);

  useEffect(() => {
    let active = true;
    mounted.current = true;
    const unlisten = listen<unknown>("extension-installs-changed", event => {
      if (!active) return;
      try { accept(event.payload); }
      catch (error) { reportError(error); }
    });
    // Subscribe before loading. A delayed list response cannot erase a newer event.
    void unlisten.then(async () => {
      if (active) await refresh();
    }).catch(reportError);
    return () => {
      active = false;
      mounted.current = false;
      cleanupTauriListener(unlisten);
    };
  }, [accept, refresh, reportError]);
  return { snapshot, loading, loadError, refresh };
}
