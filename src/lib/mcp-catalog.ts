import type { McpConnectorSpec } from "@/types/mcp";
import { MCP_CATALOG_CLOUD } from "./mcp-catalog-cloud";
import { MCP_CATALOG_LOCAL } from "./mcp-catalog-local";

export const MCP_CATALOG: McpConnectorSpec[] = [
  ...MCP_CATALOG_CLOUD,
  ...MCP_CATALOG_LOCAL,
];
