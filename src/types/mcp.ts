export type McpCategory =
  | "productivity"
  | "design"
  | "devtools"
  | "communication"
  | "ai-ml"
  | "scraping"
  | "community";

type McpAuthType = "oauth" | "token" | "none";
type McpLocale = "fr" | "en" | "es" | "de" | "it" | "zh" | "ja";

export interface McpConnectorSpec {
  id: string;
  display_name: string;
  category: McpCategory;
  auth_type: McpAuthType;
  short_descriptions: Record<McpLocale, string>;
  author: string;
  url: string;
  tools: string[];
  endpoint?: string;
  install_command?: string;
  env_keys?: string[];
  os_restrict?: "macos";
  coming_soon?: boolean;
}

type McpConnectorStatus = "connected" | "disconnected";

export interface ConfiguredMcp {
  id: string;
  status: McpConnectorStatus;
  enabled_in_chat: boolean;
  endpoint?: string;
  install_command?: string;
  env_keys?: string[];
}

export type ConfiguredMcpFull = McpConnectorSpec & ConfiguredMcp;

export function getMcpDescription(spec: McpConnectorSpec, lang: string): string {
  const base = lang.split("-")[0] as McpLocale;
  return spec.short_descriptions[base] ?? spec.short_descriptions.en;
}
