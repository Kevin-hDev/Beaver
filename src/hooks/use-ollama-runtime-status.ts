import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { OllamaRuntimeStatus } from "@/types/ollama-runtime";

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
      setStatus(await invoke<OllamaRuntimeStatus>("get_ollama_runtime_status"));
    } catch {
      setStatus(null);
      setReadError(true);
    } finally {
      setLoading(false);
    }
  }, []);
  useEffect(() => { void Promise.resolve().then(refresh); }, [refresh]);
  return { status, loading, readError, refresh };
}
