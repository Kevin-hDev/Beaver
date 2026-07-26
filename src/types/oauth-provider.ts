export type OAuthProviderId = "openai" | "moonshot" | "xai";

export interface OAuthProviderStatus {
  id: OAuthProviderId;
  display_name: string;
  connected: boolean;
  account: string | null;
  experimental: boolean;
}

export interface OAuthLoginProgress {
  provider_id: OAuthProviderId;
  stage: "starting" | "browser_open" | "device_code" | "waiting" | "success" | "cancelled" | "error";
  hint?: string;
  verification_url?: string;
  user_code?: string;
}
