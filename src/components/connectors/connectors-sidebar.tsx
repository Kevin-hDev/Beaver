import { useTranslation } from "react-i18next";
import { McpIcon } from "@/lib/mcp-icons";
import type { ConfiguredMcpFull } from "@/types/mcp";
import "./connectors-sidebar.css";

interface ConnectorsSidebarProps {
  configured: ConfiguredMcpFull[];
  selectedId: string | null;
  loadError: boolean;
  onSelect: (id: string | null) => void;
}

export function ConnectorsSidebar({
  configured,
  selectedId,
  loadError,
  onSelect,
}: ConnectorsSidebarProps) {
  const { t } = useTranslation();

  return (
    <div className="cts-sidebar">
      <div className="cts-header">{t("connectors.sidebar.configured")}</div>
      <div className="cts-list">
        {configured.length === 0 ? (
          <div className="cts-empty">
            {t(loadError ? "connectors.sidebar.loadError" : "connectors.sidebar.empty")}
          </div>
        ) : (
          configured.map((c) => (
            <button
              key={c.id}
              type="button"
              className={`cts-item ${selectedId === c.id ? "active" : ""}`}
              onClick={() => onSelect(c.id)}
            >
              <McpIcon connectorId={c.id} displayName={c.display_name} size="var(--icon-lg)" />
              <span>{c.display_name}</span>
              {c.status === "disconnected" && <span className="cts-dot-off" />}
            </button>
          ))
        )}
      </div>
    </div>
  );
}
