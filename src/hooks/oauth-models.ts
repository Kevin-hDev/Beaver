import { invoke } from "@tauri-apps/api/core";
import type { AvailableModel } from "./use-available-models";
import type { ReasoningMode } from "@/lib/reasoning-modes";

export interface OAuthModelInfo {
  id: string;
  provider_id: "openai" | "moonshot" | "xai";
  display_name: string;
  context_length?: number;
  supports_tools: boolean;
  supports_vision: boolean;
  supports_thinking: boolean;
  reasoning_modes?: ReasoningMode[];
  default_reasoning_mode?: ReasoningMode;
  interactive_only: boolean;
}

export type OAuthProviderIssueCode =
  | "moonshot_membership_unverified"
  | "xai_subscription_or_credits_required"
  | "oauth_reauthentication_required"
  | "rate_limit"
  | "provider_access_unavailable"
  | "model_catalog_unavailable";

export interface OAuthModelsResponse {
  models: OAuthModelInfo[];
  issues: Array<{ provider_id: OAuthModelInfo["provider_id"]; code: OAuthProviderIssueCode }>;
}

export interface OAuthModelsResult {
  groups: Map<string, AvailableModel[]>;
  issues: Map<OAuthModelInfo["provider_id"], OAuthProviderIssueCode>;
}

const PROVIDERS = {
  openai: { id: "codex-oauth", name: "OpenAI · OAuth" },
  moonshot: { id: "moonshot-oauth", name: "Moonshot AI · OAuth" },
  xai: { id: "xai-oauth", name: "xAI · OAuth" },
} as const;

export function mapOAuthModels(models: OAuthModelInfo[]): Map<string, AvailableModel[]> {
  const groups = new Map<string, AvailableModel[]>();
  for (const model of models) {
    const provider = PROVIDERS[model.provider_id];
    if (!provider || typeof model.id !== "string" || model.id.length === 0 || model.id.length > 128) continue;
    const mapped: AvailableModel = {
      id: model.id,
      display_name: model.display_name,
      provider_id: provider.id,
      provider_name: provider.name,
      auth_source: "oauth",
      is_local: false,
      supports_tools: model.supports_tools,
      supports_vision: model.supports_vision,
      supports_thinking: model.supports_thinking,
      reasoning_modes: model.reasoning_modes,
      default_reasoning_mode: model.default_reasoning_mode,
      is_free: true,
      interactive_only: model.interactive_only,
      hint: model.context_length ? `${Math.round(model.context_length / 1000)}K ctx` : undefined,
    };
    groups.set(provider.id, [...(groups.get(provider.id) ?? []), mapped]);
  }
  return groups;
}

const ISSUE_CODES = new Set<OAuthProviderIssueCode>([
  "moonshot_membership_unverified", "xai_subscription_or_credits_required",
  "oauth_reauthentication_required", "rate_limit",
  "provider_access_unavailable", "model_catalog_unavailable",
]);
const CACHE_MS = 15_000;
export const OAUTH_MODELS_UPDATED_EVENT = "cl-go:oauth-models-updated";
let cached: { value: OAuthModelsResult; at: number } | null = null;
let pending: Promise<OAuthModelsResult> | null = null;

export function mapOAuthResponse(response: OAuthModelsResponse): OAuthModelsResult {
  const models = Array.isArray(response.models) ? response.models.slice(0, 600) : [];
  const issues = new Map<OAuthModelInfo["provider_id"], OAuthProviderIssueCode>();
  if (Array.isArray(response.issues)) {
    for (const issue of response.issues.slice(0, 3)) {
      if (PROVIDERS[issue.provider_id] && ISSUE_CODES.has(issue.code)) {
        issues.set(issue.provider_id, issue.code);
      }
    }
  }
  return { groups: mapOAuthModels(models), issues };
}

export function invalidateOAuthModelsCache() {
  cached = null;
}

export function notifyOAuthModelsChanged() {
  window.dispatchEvent(new Event(OAUTH_MODELS_UPDATED_EVENT));
}

export async function fetchOAuthModels(force = false): Promise<OAuthModelsResult> {
  if (!force && cached && Date.now() - cached.at < CACHE_MS) return cached.value;
  pending ??= invoke<OAuthModelsResponse>("list_oauth_provider_models")
    .then(mapOAuthResponse)
    .then((value) => {
      cached = { value, at: Date.now() };
      return value;
    })
    .finally(() => { pending = null; });
  return pending;
}
