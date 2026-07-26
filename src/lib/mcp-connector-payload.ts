import type { ConfiguredMcp, McpConnectorSpec } from "@/types/mcp";

export function connectorPayload(spec: McpConnectorSpec): ConfiguredMcp {
  return {
    id: spec.id,
    status: "connected",
    enabled_in_chat: true,
    endpoint: spec.endpoint,
    install_command: spec.install_command,
    env_keys: spec.env_keys,
  };
}
