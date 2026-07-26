type ChannelStatus = "off" | "starting" | "running" | "error" | "stopping";

export interface ChannelHealthEntry {
  channel_id: string;
  account_id: string;
  status: ChannelStatus;
  error?: string;
}

export interface GatewayHealth {
  running: boolean;
  channels: ChannelHealthEntry[];
}

export interface ChannelAccountConfig {
  account_id: string;
  enabled: boolean;
  allowlist: string[];
  require_mention: boolean;
  provider: string;
  model: string;
}

export interface GatewayConfig {
  enabled: boolean;
  start_with_app: boolean;
  run_when_window_closed: boolean;
  default_provider: string;
  default_model: string;
  max_sessions: number;
  message_max_chars: number;
  rate_limits: RateLimitConfig;
  audit: AuditConfig;
  channels: ChannelsConfig;
}

interface RateLimitConfig {
  per_user_per_minute: number;
  per_channel_per_minute: number;
  global_per_minute: number;
}

interface AuditConfig {
  enabled: boolean;
  retention_days: number;
}

interface ChannelsConfig {
  telegram: ChannelAccountConfig[];
  slack: ChannelAccountConfig[];
  discord: ChannelAccountConfig[];
}

export type ChannelType = "telegram" | "slack" | "discord";
