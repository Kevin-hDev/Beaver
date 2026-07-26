import type { OAuthProviderId } from "@/types/oauth-provider";

const OAUTH_USAGE_LINKS: Record<OAuthProviderId, string> = {
  openai: "https://chatgpt.com/codex/settings/usage",
  moonshot: "https://www.kimi.com/code/console",
  xai: "https://grok.com/?_s=usage",
};

export function oauthUsageLink(providerId: OAuthProviderId): string {
  return OAUTH_USAGE_LINKS[providerId];
}

export function oauthUsageConnectionId(providerId: OAuthProviderId): string {
  if (providerId === "openai") return "codex-oauth";
  if (providerId === "moonshot") return "moonshot-oauth";
  return "xai-oauth";
}
