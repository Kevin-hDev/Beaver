import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { OllamaModel } from "@/types/agent";
import type { AvailableModel } from "./available-model-types";
import { fetchCloudModels, type ModelCatalogIssue } from "./cloud-models";
import {
  fetchOAuthModels, invalidateOAuthModelsCache, mapOAuthModels, mapOAuthResponse,
  OAUTH_MODELS_UPDATED_EVENT,
} from "./oauth-models";

export { mapOAuthModels, mapOAuthResponse };
export type { AvailableModel } from "./available-model-types";

let cachedGroups: Map<string, AvailableModel[]> = new Map();
let cachedIssues: Map<string, ModelCatalogIssue> = new Map();
let pendingFetchAll: Promise<AvailableModelsResult> | null = null;

interface AvailableModelsResult {
  groups: Map<string, AvailableModel[]>;
  issues: Map<string, ModelCatalogIssue>;
}

async function fetchOllamaModels(): Promise<AvailableModel[]> {
  const ollamaModels = await invoke<OllamaModel[]>("list_ollama_models");
  return ollamaModels.map(
    (m): AvailableModel => ({
      id: m.name,
      provider_id: "ollama",
      provider_name: "Ollama",
      auth_source: "local",
      is_local: true,
      supports_tools: m.capabilities?.includes("tools") ?? false,
      supports_vision: m.capabilities?.includes("vision") ?? false,
      supports_thinking: m.capabilities?.includes("thinking") ?? false,
      reasoning_modes: m.reasoning_modes,
      reasoning_contract: undefined,
      default_reasoning_mode: m.default_reasoning_mode ?? undefined,
      context_length: m.context_length,
      context_usage_includes_reasoning: m.context_usage_includes_reasoning,
      hint: m.parameter_size,
    }),
  );
}

async function fetchAllModels(): Promise<AvailableModelsResult> {
  const result = new Map<string, AvailableModel[]>();
  let issues = new Map<string, ModelCatalogIssue>();

  const [ollamaResult, cloudResult, oauthResult] = await Promise.allSettled([
    fetchOllamaModels(),
    fetchCloudModels(),
    fetchOAuthModels(),
  ]);

  if (ollamaResult.status === "fulfilled" && ollamaResult.value.length > 0) {
    result.set("ollama", ollamaResult.value);
  }
  if (cloudResult.status === "fulfilled") {
    for (const [k, v] of cloudResult.value.groups) result.set(k, v);
    issues = cloudResult.value.issues;
  }
  if (oauthResult.status === "fulfilled") {
    for (const [key, models] of oauthResult.value.groups) result.set(key, models);
  }

  cachedGroups = result;
  cachedIssues = issues;
  return { groups: result, issues };
}

function getAllModels(): Promise<AvailableModelsResult> {
  pendingFetchAll ??= fetchAllModels().finally(() => {
    pendingFetchAll = null;
  });
  return pendingFetchAll;
}

export function useAvailableModels() {
  const [groups, setGroups] = useState<Map<string, AvailableModel[]>>(cachedGroups);
  const [issues, setIssues] = useState<Map<string, ModelCatalogIssue>>(cachedIssues);
  const [loading, setLoading] = useState(cachedGroups.size === 0);

  const refresh = useCallback(async () => {
    const result = await getAllModels();
    setGroups(result.groups);
    setIssues(result.issues);
    setLoading(false);
  }, []);

  const refreshOllama = useCallback(async () => {
    try {
      const ollama = await fetchOllamaModels();
      setGroups((prev) => {
        const next = new Map(prev);
        if (ollama.length > 0) next.set("ollama", ollama);
        else next.delete("ollama");
        cachedGroups = next;
        return next;
      });
    } catch {
      setGroups((prev) => {
        const next = new Map(prev);
        next.delete("ollama");
        cachedGroups = next;
        return next;
      });
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
    const unsubOllama = listen("ollama-models-changed", () => void refreshOllama());
    const unsubFs = listen("fs:config-changed", () => void refresh());
    const unsubOAuth = listen("oauth-provider-status-changed", () => {
      invalidateOAuthModelsCache();
      void refresh();
    });
    const refreshOAuthModels = () => { void refresh(); };
    window.addEventListener(OAUTH_MODELS_UPDATED_EVENT, refreshOAuthModels);
    const unsubStatus = listen<boolean>("ollama-status", (e) => {
      if (e.payload) setTimeout(() => void refreshOllama(), 2000);
    });
    return () => {
      cleanupTauriListener(unsubOllama);
      cleanupTauriListener(unsubFs);
      cleanupTauriListener(unsubOAuth);
      cleanupTauriListener(unsubStatus);
      window.removeEventListener(OAUTH_MODELS_UPDATED_EVENT, refreshOAuthModels);
    };
  }, [refresh, refreshOllama]);

  return { groups, issues, loading, refresh };
}

export function withoutInteractiveOnlyModels(
  groups: Map<string, AvailableModel[]>,
): Map<string, AvailableModel[]> {
  const filtered = new Map<string, AvailableModel[]>();
  for (const [providerId, models] of groups) {
    const allowed = models.filter((model) => !model.interactive_only);
    if (allowed.length > 0) filtered.set(providerId, allowed);
  }
  return filtered;
}
