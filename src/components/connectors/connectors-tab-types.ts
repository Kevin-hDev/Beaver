import type { McpConnectorSpec } from "@/types/mcp";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";

export type DialogState =
  | { kind: "none" }
  | { kind: "browse" }
  | { kind: "config"; connector: McpConnectorSpec; returnTo: "browse" | "none" }
  | { kind: "oauth-pending"; connector: McpConnectorSpec; returnTo: "browse" | "none" }
  | { kind: "confirm-add"; connector: McpConnectorSpec; returnTo: "browse" | "none" }
  | { kind: "confirm-disconnect"; connectorId: string };

export interface ConnectorsTabProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}
