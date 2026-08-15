import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import type { RetryIndicatorState } from "@/types/agent";
import type { OllamaRuntimeStatus } from "@/types/ollama-runtime";
import { parseOllamaRuntimeStatus } from "@/lib/ollama-runtime-status";

const MAX_CONNECTION_RETRIES = 10;
const CONNECTION_RETRY_DELAY_MS = 4000;

interface OllamaConnectionRetryOptions {
  error?: string;
  isConnectionError?: boolean;
  isStreaming: boolean;
  onRetry: () => void;
}

interface OllamaConnectionRetryResult {
  indicator: RetryIndicatorState | null;
  suppressError: boolean;
}

type RetryPhase = {
  error: string | null;
  kind: "idle" | "retrying" | "resolved" | "exhausted";
};

export function useOllamaConnectionRetry({
  error,
  isConnectionError,
  isStreaming,
  onRetry,
}: OllamaConnectionRetryOptions): OllamaConnectionRetryResult {
  const [indicator, setIndicator] = useState<RetryIndicatorState | null>(null);
  const [phase, setPhase] = useState<RetryPhase>({ error: null, kind: "idle" });
  const retryRef = useRef(onRetry);
  const runningRef = useRef(false);
  const terminalRef = useRef<{ error: string; kind: "resolved" | "exhausted" } | null>(null);

  useEffect(() => {
    retryRef.current = onRetry;
  }, [onRetry]);

  useEffect(() => {
    if (isStreaming) {
      terminalRef.current = null;
      return;
    }

    if (!error || !isConnectionError) return;

    if (runningRef.current || terminalRef.current?.error === error) return;

    let cancelled = false;
    runningRef.current = true;

    const run = async () => {
      setPhase({ error, kind: "retrying" });
      for (let attempt = 1; attempt <= MAX_CONNECTION_RETRIES; attempt += 1) {
        if (cancelled) break;
        setIndicator({
          reasonKey: "agentLocal.retry.connection",
          attempt,
          maxAttempts: MAX_CONNECTION_RETRIES,
        });

        const status = parseOllamaRuntimeStatus(
          await invoke<unknown>("get_ollama_runtime_status").catch(() => null),
        );
        if (cancelled) break;
        if (hasAvailableDaemon(status)) {
          setIndicator(null);
          terminalRef.current = { error, kind: "resolved" };
          setPhase({ error, kind: "resolved" });
          runningRef.current = false;
          retryRef.current();
          return;
        }

        if (attempt < MAX_CONNECTION_RETRIES) {
          await wait(CONNECTION_RETRY_DELAY_MS);
        }
      }

      if (!cancelled) {
        terminalRef.current = { error, kind: "exhausted" };
        setPhase({ error, kind: "exhausted" });
        setIndicator(null);
        runningRef.current = false;
      }
    };

    void run();

    return () => {
      cancelled = true;
      runningRef.current = false;
    };
  }, [error, isConnectionError, isStreaming]);

  if (isStreaming || !error || !isConnectionError) {
    return { indicator: null, suppressError: false };
  }

  const currentKind = phase.error === error ? phase.kind : "idle";
  if (currentKind === "exhausted") {
    return { indicator: null, suppressError: false };
  }
  if (currentKind === "resolved") {
    return { indicator: null, suppressError: true };
  }

  return {
    indicator: phase.error === error && indicator ? indicator : {
      reasonKey: "agentLocal.retry.connection",
      attempt: 1,
      maxAttempts: MAX_CONNECTION_RETRIES,
    },
    suppressError: true,
  };
}

function hasAvailableDaemon(status: OllamaRuntimeStatus | null): boolean {
  return status !== null && status.daemon !== "unavailable";
}

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
