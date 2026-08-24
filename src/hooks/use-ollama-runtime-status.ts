import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { OllamaRuntimeStatus } from "@/types/ollama-runtime";
import { parseOllamaRuntimeStatus } from "@/lib/ollama-runtime-status";

export interface OllamaRuntimeStatusResult {
  status: OllamaRuntimeStatus | null;
  loading: boolean;
  readError: boolean;
  refresh: () => Promise<void>;
}

export function useOllamaRuntimeStatus(): OllamaRuntimeStatusResult {
  const [status, setStatus] = useState<OllamaRuntimeStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [readError, setReadError] = useState(false);
  const refresh = useCallback(async () => {
    setLoading(true);
    setReadError(false);
    try {
      const status = parseOllamaRuntimeStatus(await invoke<unknown>("get_ollama_runtime_status"));
      if (!status) throw new Error("invalid runtime status");
      setStatus(status);
    } catch {
      setStatus(null);
      setReadError(true);
    } finally {
      setLoading(false);
    }
  }, []);
  useEffect(() => {
    let active = true;
    const unlisten = listen<boolean>("ollama-status", () => { void refresh(); });
    // Armer l'écoute avant la lecture ferme la course avec un démarrage Ollama
    // qui se termine pendant le montage du panneau.
    void unlisten.then(
      () => { if (active) void refresh(); },
      () => { if (active) void refresh(); },
    );
    return () => {
      active = false;
      cleanupTauriListener(unlisten);
    };
  }, [refresh]);
  return { status, loading, readError, refresh };
}
