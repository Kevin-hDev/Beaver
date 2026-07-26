import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { detectLocalSites } from "./browser-ipc";
import { parseLocalSiteScan, type LocalSiteScan } from "./browser-types";

const LOCAL_SITE_SCAN_INTERVAL_MS = 5_000;
const LOCAL_SITES_EVENT = "browser-local-sites-changed-v1";
const EMPTY_SCAN: LocalSiteScan = { sites: [], generation: 0, changed: false };

export function useLocalSites(homeVisible: boolean) {
  const [scan, setScan] = useState<LocalSiteScan>(EMPTY_SCAN);
  const [error, setError] = useState(false);
  const runningRef = useRef(false);

  const accept = useCallback((next: LocalSiteScan) => {
    setScan((current) => next.generation > current.generation ? next : current);
  }, []);

  useEffect(() => {
    if (!homeVisible) return;
    let cancelled = false;
    const run = async () => {
      if (runningRef.current) return;
      runningRef.current = true;
      try {
        const next = await detectLocalSites(true);
        if (!cancelled) {
          accept(next);
          setError(false);
        }
      } catch {
        if (!cancelled) setError(true);
      } finally {
        runningRef.current = false;
      }
    };
    void run();
    const timer = window.setInterval(() => { void run(); }, LOCAL_SITE_SCAN_INTERVAL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [accept, homeVisible]);

  useEffect(() => {
    if (!homeVisible) return;
    const unlisten = listen<unknown>(LOCAL_SITES_EVENT, (event) => {
      const next = parseLocalSiteScan(event.payload);
      if (next) accept(next);
    });
    return () => cleanupTauriListener(unlisten);
  }, [accept, homeVisible]);

  return { sites: scan.sites, generation: scan.generation, error };
}
