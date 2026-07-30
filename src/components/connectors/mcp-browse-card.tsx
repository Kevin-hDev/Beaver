import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { Plus, Check, ArrowSquareOut } from "@/components/ui/icons";
import { McpIcon } from "@/lib/mcp-icons";
import { getMcpDescription } from "@/types/mcp";
import type { McpConnectorSpec } from "@/types/mcp";
import "@/components/ui/browse-card.css";
import "./mcp-browse-card.css";

interface McpBrowseCardProps {
  connector: McpConnectorSpec;
  configured: boolean;
  onAdd: () => void;
}

export function McpBrowseCard({ connector, configured, onAdd }: McpBrowseCardProps) {
  const { t, i18n } = useTranslation();
  const locked = connector.coming_soon === true;

  const handleLinkClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    void open(connector.url);
  };

  return (
    <div className={`browse-card ${configured ? "is-configured" : ""} ${locked ? "is-locked" : ""}`}>
      <McpIcon connectorId={connector.id} displayName={connector.display_name} size={40} />
      <div className="browse-card-body">
        <div className="browse-card-name">
          {connector.display_name}
          {locked && <span className="browse-chip mcbc-soon">{t("connectors.comingSoon")}</span>}
        </div>
        <div className="browse-card-desc">{getMcpDescription(connector, i18n.language)}</div>
        <div className="browse-card-meta">
          <span className="browse-chip browse-chip-cat">{connector.category.toUpperCase()}</span>
          <span className="browse-chip">{connector.author}</span>
          <button
            type="button"
            className="browse-card-link mcbc-link"
            onClick={handleLinkClick}
            title={connector.url}
          >
            <ArrowSquareOut size="var(--icon-xs)" />
          </button>
        </div>
      </div>
      <button
        type="button"
        className={`icon-btn browse-card-action ${configured ? "done" : ""}`}
        onClick={configured || locked ? undefined : onAdd}
        disabled={configured || locked}
      >
        {configured ? <Check size="var(--icon-md)" weight="bold" /> : <Plus size="var(--icon-md)" weight="bold" />}
      </button>
    </div>
  );
}
