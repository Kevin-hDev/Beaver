import { invoke } from "@tauri-apps/api/core";
import type { AvailableModel } from "./available-model-types";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import type { OAuthProviderId } from "@/types/oauth-provider";

export interface OAuthModelInfo {
  id: string;
  provider_id: string;
  connection_id: string;
  provider_display_name: string;
  display_name: string;
  context_length?: number;
  supports_tools: boolean;
  supports_vision: boolean;
  supports_thinking: boolean;
  supports_fast_mode: boolean;
  reasoning_modes?: ReasoningMode[];
  default_reasoning_mode?: ReasoningMode;
  context_usage_includes_reasoning: boolean;
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
  issues: Array<{ provider_id: OAuthProviderId; code: OAuthProviderIssueCode }>;
}

export interface OAuthModelsResult {
  groups: Map<string, AvailableModel[]>;
  issues: Map<OAuthProviderId, OAuthProviderIssueCode>;
}

export function mapOAuthModels(models: OAuthModelInfo[]): Map<string, AvailableModel[]> {
  const groups = new Map<string, AvailableModel[]>();
  for (const model of models) {
    if (!validPublicIdentifier(model.connection_id, 32)
      || !validDisplayName(model.provider_display_name)
      || !validPublicIdentifier(model.id, 128)) continue;
    const mapped: AvailableModel = {
      id: model.id,
      display_name: model.display_name,
      provider_id: model.connection_id,
      provider_name: `${model.provider_display_name} · OAuth`,
      auth_source: "oauth",
      is_local: false,
      supports_tools: model.supports_tools,
      supports_vision: model.supports_vision,
      supports_thinking: model.supports_thinking,
      supports_fast_mode: model.supports_fast_mode,
      reasoning_modes: model.reasoning_modes,
      default_reasoning_mode: model.default_reasoning_mode,
      context_length: model.context_length,
      context_usage_includes_reasoning: model.context_usage_includes_reasoning,
      is_free: false,
      interactive_only: model.interactive_only,
      hint: model.context_length ? `${Math.round(model.context_length / 1000)}K ctx` : undefined,
    };
    groups.set(model.connection_id, [...(groups.get(model.connection_id) ?? []), mapped]);
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
  const issues = new Map<OAuthProviderId, OAuthProviderIssueCode>();
  if (Array.isArray(response.issues)) {
    for (const issue of response.issues.slice(0, 3)) {
      if (ISSUE_CODES.has(issue.code)) {
        issues.set(issue.provider_id, issue.code);
      }
    }
  }
  return { groups: mapOAuthModels(models), issues };
}

function validPublicIdentifier(value: unknown, maxLength: number): value is string {
  return typeof value === "string" && value.length > 0 && value.length <= maxLength
    && /^[A-Za-z0-9._:/-]+$/.test(value) && !value.includes("..");
}

function validDisplayName(value: unknown): value is string {
  return typeof value === "string" && value.length > 0 && value.length <= 80
    && Array.from(value).every((character) => {
      const code = character.charCodeAt(0);
      return code >= 32 && code !== 127;
    });
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
