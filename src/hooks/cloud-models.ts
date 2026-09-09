import { invoke } from "@tauri-apps/api/core";
import { isKnownAgentErrorCode } from "@/lib/agent-error-codes";
import type { ProviderSpec } from "@/types/api";
import type { AvailableModel, LlmModelInfo } from "./available-model-types";

export interface ModelCatalogIssue {
  providerName: string;
  code: string;
}

export interface CloudModelsResult {
  groups: Map<string, AvailableModel[]>;
  issues: Map<string, ModelCatalogIssue>;
}

type CloudModelSettlement =
  | { status: "fulfilled"; value: LlmModelInfo[] }
  | { status: "rejected"; reason: unknown };

export async function fetchCloudModels(): Promise<CloudModelsResult> {
  const [catalogResult, configuredResult] = await Promise.allSettled([
    invoke<ProviderSpec[]>("list_llm_providers_catalog"),
    invoke<string[]>("list_configured_providers"),
  ]);
  if (configuredResult.status === "rejected") {
    throw new Error("configured providers unavailable");
  }
  const configuredIds = configuredResult.value;
  if (catalogResult.status === "rejected") {
    return {
      groups: new Map(),
      issues: new Map(configuredIds.map((providerId) => [providerId, {
        providerName: providerId,
        code: "model_catalog_unavailable",
      }])),
    };
  }
  const catalog = catalogResult.value;
  const configured = catalog.filter((spec) => configuredIds.includes(spec.id));
  const results = await Promise.allSettled(
    configured.map((spec) => invoke<LlmModelInfo[]>("list_llm_models", {
      providerId: spec.id,
    })),
  );
  return mapCloudModelSettlements(configured, results);
}

export function mapCloudModelSettlements(
  configured: readonly ProviderSpec[],
  results: readonly CloudModelSettlement[],
): CloudModelsResult {
  const groups = new Map<string, AvailableModel[]>();
  const issues = new Map<string, ModelCatalogIssue>();
  for (const [index, spec] of configured.entries()) {
    const entry = results[index];
    if (!entry || entry.status === "rejected") {
      const reason = entry?.status === "rejected" ? entry.reason : undefined;
      const code = typeof reason === "string" && isKnownAgentErrorCode(reason)
        ? reason
        : "model_catalog_unavailable";
      issues.set(spec.id, { providerName: spec.display_name, code });
      continue;
    }
    const models = entry.value.map((model): AvailableModel => ({
      id: model.id,
      display_name: model.display_name,
      provider_id: spec.id,
      provider_name: spec.display_name,
      auth_source: "api",
      is_local: false,
      supports_tools: model.supports_tools,
      supported_parameters: model.supported_parameters,
      supports_vision: model.supports_vision ?? false,
      supports_thinking: model.supports_thinking ?? false,
      supports_fast_mode: model.supports_fast_mode,
      reasoning_modes: model.reasoning_modes,
      reasoning_contract: model.reasoning_contract,
      default_reasoning_mode: model.default_reasoning_mode,
      context_length: model.context_length,
      context_usage_includes_reasoning: model.context_usage_includes_reasoning,
      is_free: model.is_free ?? false,
      hint: model.context_length
        ? `${Math.round(model.context_length / 1000)}K ctx`
        : undefined,
    }));
    if (models.length > 0) groups.set(spec.id, models);
  }
  return { groups, issues };
}
