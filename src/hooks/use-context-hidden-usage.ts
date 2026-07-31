import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ContextUsageOptions } from "./context-usage-breakdown";

interface HiddenContextUsage {
  systemPromptTokens: number;
  metaContextTokens: number;
  skillContextTokens: number;
  memoryContextTokens: number;
  systemToolDefinitionTokens: number;
  mcpDefinitionTokens: number;
}

interface UseContextHiddenUsageArgs {
  enabled?: boolean;
  sessionId: string;
  model: string;
  provider: string;
  workingDir?: string;
  permissionMode?: string;
  planMode?: boolean;
  supportsTools?: boolean;
}

export function useContextHiddenUsage({
  enabled = true,
  sessionId,
  model,
  provider,
  workingDir,
  permissionMode,
  planMode,
  supportsTools,
}: UseContextHiddenUsageArgs): ContextUsageOptions {
  const [usage, setUsage] = useState<ContextUsageOptions>({});

  useEffect(() => {
    let alive = true;
    if (!enabled || !sessionId || !model) {
      queueMicrotask(() => {
        if (alive) setUsage({});
      });
      return;
    }
    invoke<HiddenContextUsage>("estimate_context_hidden_usage", {
      sessionId,
      model,
      provider,
      workingDir: workingDir ?? null,
      permissionMode: permissionMode ?? null,
      planMode: planMode ?? null,
      supportsTools: supportsTools ?? null,
    })
      .then((result) => {
        if (!alive) return;
        setUsage({
          systemPromptTokens: result.systemPromptTokens,
          metaContextTokens: result.metaContextTokens,
          skillContextTokens: result.skillContextTokens,
          memoryContextTokens: result.memoryContextTokens,
          systemToolDefinitionTokens: result.systemToolDefinitionTokens,
          mcpDefinitionTokens: result.mcpDefinitionTokens,
        });
      })
      .catch(() => {
        if (alive) setUsage({});
      });
    return () => {
      alive = false;
    };
  }, [enabled, sessionId, model, provider, workingDir, permissionMode, planMode, supportsTools]);

  return usage;
}
