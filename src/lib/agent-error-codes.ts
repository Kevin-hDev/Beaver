export const KNOWN_ERROR_KEYS: Readonly<Record<string, string>> = {
  ollama_connection_lost: "errors.ollamaConnectionLost",
  model_not_found: "errors.modelNotFound",
  rate_limit: "errors.rateLimited",
  auth_failed: "errors.authFailed",
  moonshot_membership_unverified: "errors.moonshotMembershipUnverified",
  xai_subscription_or_credits_required: "errors.xaiSubscriptionOrCreditsRequired",
  provider_access_unavailable: "errors.providerAccessUnavailable",
  provider_quota_exhausted: "errors.providerQuotaExhausted",
  stream_interrupted: "errors.streamInterrupted",
  provider_connection_failed: "errors.providerConnectionFailed",
  provider_temporarily_unavailable: "errors.providerTemporarilyUnavailable",
  provider_request_rejected: "errors.providerRequestRejected",
  provider_payload_too_large: "errors.providerPayloadTooLarge",
  provider_configuration_invalid: "errors.providerConfigurationInvalid",
  oauth_reauthentication_required: "errors.oauthReauthenticationRequired",
  ollama_server_error: "errors.ollamaServerError",
  connection_lost: "errors.connectionLost",
  timeout: "errors.timeout",
  provider_overloaded: "errors.providerOverloaded",
  provider_error: "errors.providerError",
  max_turns: "errors.maxTurns",
  circuit_breaker: "errors.circuitBreaker",
  tool_error: "errors.toolError",
  stream_error: "errors.streamError",
};

export function isKnownAgentErrorCode(code: string): boolean {
  return Object.prototype.hasOwnProperty.call(KNOWN_ERROR_KEYS, code);
}
