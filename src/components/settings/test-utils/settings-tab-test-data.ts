export function provider(id: string, displayName: string, category: string) {
  return {
    id,
    display_name: displayName,
    category,
    signup_url: "",
  };
}

export function agentSettings() {
  return {
    permission_mode: "auto",
    enabled_optional_tools: ["load_skill", "plan_mode"],
  };
}

export function memoryOverview() {
  return {
    settings: { mode: "disabled", contextBudgetTokens: 3000 },
    global: {
      id: "global",
      label: "Global",
      topicCount: 1,
      totalBytes: 512,
      lastUpdated: "2026-07-24T20:10:00Z",
      topics: [{
        id: "019f951b-38a1-7882-bf2f-0784e266c911",
        title: "Interface compacte",
        summary: "Préférence pour une interface compacte.",
        memoryType: "preference",
        status: "confirmed",
        tags: ["ui"],
        createdAt: "2026-07-24T20:00:00Z",
        updatedAt: "2026-07-24T20:10:00Z",
        source: "user",
        sessionId: "019f951b-38a1-7882-bf2f-0784e266c911",
        path: "/memory/global/topics/interface.md",
      }],
      topicsLoaded: true,
    },
    activeProject: undefined,
    otherProjects: [{
      id: "bbbbbbbbbbbbbbbbbbbbbbbb",
      label: "Autre projet",
      topicCount: 1,
      totalBytes: 256,
      lastUpdated: "2026-07-23T18:00:00Z",
      topics: [],
      topicsLoaded: false,
    }],
    legacyDetected: false,
  };
}

export function agentToolCatalog() {
  return [
    { id: "bash", locked: true, defaultEnabled: true, group: "core" },
    { id: "search_mcp_tools", locked: true, defaultEnabled: true, group: "mcp" },
    { id: "load_skill", locked: false, defaultEnabled: true, group: "workflow" },
    { id: "forecast_run", locked: false, defaultEnabled: false, group: "forecast" },
  ];
}

export function agentToolGroups() {
  return [
    { id: "web", locked: true, defaultEnabled: true, toolIds: ["web_search", "web_fetch"] },
    { id: "skills", locked: false, defaultEnabled: true, toolIds: ["load_skill"] },
    { id: "plan_mode", locked: false, defaultEnabled: true, toolIds: ["plan_mode"] },
    {
      id: "forecast",
      locked: false,
      defaultEnabled: false,
      toolIds: [
        "forecast_data_audit",
        "forecast_run",
        "forecast_models",
        "forecast_analyze",
        "forecast_read",
      ],
    },
  ];
}

export function ollamaModels() {
  return [{
    name: "llama3.2:latest",
    size: 2000,
    family: "llama",
    parameter_size: "3B",
    quantization: "Q4_K_M",
    architecture: "llama",
    is_moe: false,
    context_length: 8192,
    capabilities: ["completion"],
    digest_short: "abc123",
    aliases: [],
    is_customized: false,
  }];
}

export function mcpConnectors() {
  return [
    { id: "canva", status: "connected", enabled_in_chat: true },
    { id: "github", status: "disconnected", enabled_in_chat: false },
  ];
}

export function gatewayStatus() {
  return {
    running: true,
    channels: [{ channel_id: "telegram", account_id: "test-telegram", ok: true }],
  };
}

export function gatewayConfig() {
  return {
    enabled: false,
    start_with_app: true,
    run_when_window_closed: true,
    default_provider: "",
    default_model: "",
    max_sessions: 500,
    message_max_chars: 8000,
    rate_limits: { per_user_per_minute: 12, per_channel_per_minute: 120, global_per_minute: 300 },
    audit: { enabled: true, retention_days: 30 },
    channels: {
      telegram: [{ account_id: "test-telegram", enabled: true, allowlist: [], require_mention: true }],
      slack: [],
      discord: [{ account_id: "test-discord", enabled: true, allowlist: [], require_mention: true }],
    },
  };
}

export function forecastModels() {
  return {
    providers: [{ id: "nixtla", display_name: "Nixtla", configured: true }],
    configured_provider_ids: ["nixtla"],
    models: [{
      id: "chronos-bolt-small",
      provider_id: "local",
      family_id: "chronos-bolt",
      display_name: "Chronos Bolt Small",
      params: "small",
      size_mb: 120,
      ram_mb: 512,
      vram_mb: null,
      cpu_supported: true,
      gpu_supported: false,
      multivariate: false,
      covariates: false,
      horizon_max: 64,
      frequencies: "D,H",
      is_cloud: false,
      installed: true,
      runtime_ready: true,
      runnable: true,
    }],
  };
}
