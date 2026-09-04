import { useId } from "react";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { McpIcon } from "@/lib/mcp-icons";

interface ChatPlusConnectorRowProps {
  connectorId: string;
  displayName: string;
  enabled: boolean;
  onToggle: () => void;
}

export function ChatPlusConnectorRow({
  connectorId,
  displayName,
  enabled,
  onToggle,
}: ChatPlusConnectorRowProps) {
  const switchId = useId();

  return (
    <div className="menu-row cpm-sub-item">
      <McpIcon
        connectorId={connectorId}
        displayName={displayName}
        size="var(--icon-lg)"
      />
      <label
        className={enabled ? "cpm-connector-label" : "cpm-connector-label cpm-disabled"}
        htmlFor={switchId}
      >
        {displayName}
      </label>
      <ToggleSwitch
        id={switchId}
        checked={enabled}
        ariaLabel={displayName}
        className="cpm-connector-switch"
        onCheckedChange={onToggle}
      />
    </div>
  );
}
