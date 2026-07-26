import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { Trash, ArrowSquareOut } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { McpIcon, mcpHasTextIcon } from "@/lib/mcp-icons";
import { SettingsCard } from "@/components/settings/settings-card";
import { getMcpDescription } from "@/types/mcp";
import type { ConfiguredMcpFull } from "@/types/mcp";
import "./connectors-detail.css";

interface ConnectorsDetailProps {
  connector: ConfiguredMcpFull;
  onToggleStatus: () => void;
  onDelete: () => Promise<void>;
}

export function ConnectorsDetail({ connector, onToggleStatus, onDelete }: ConnectorsDetailProps) {
  const { t, i18n } = useTranslation();
  const isConnected = connector.status === "connected";
  const hasText = mcpHasTextIcon(connector.id);
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    if (!confirmDelete) return;
    const timer = setTimeout(() => setConfirmDelete(false), 5000);
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        setConfirmDelete(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      clearTimeout(timer);
      window.removeEventListener("keydown", onKey);
    };
  }, [confirmDelete]);

  return (
    <div className="ctd-scroll">
      <div className="ctd-inner">
        <div className="ctd-header">
          <div className="ctd-info">
            {hasText ? (
              <McpIcon connectorId={connector.id} displayName={connector.display_name} size={40} variant="text" textWidth />
            ) : (
              <>
                <McpIcon connectorId={connector.id} displayName={connector.display_name} size={36} variant="text" />
                <h2 className="ctd-name">{connector.display_name}</h2>
              </>
            )}
          </div>
          <div className="ctd-actions">
            <button
              type="button"
              className={`ctd-status-btn ${isConnected ? "connected" : "disconnected"}`}
              onClick={onToggleStatus}
            >
              <span className="ctd-status-dot" />
              {t(isConnected ? "connectors.detail.connected" : "connectors.detail.disconnected")}
            </button>
            <Tooltip label={t("connectors.detail.confirmDeleteBtn")} align="right">
              <button type="button" className="ak-icon-btn danger" onClick={() => setConfirmDelete(true)}>
                <Trash size="var(--icon-md)" />
              </button>
            </Tooltip>
          </div>
        </div>

        <SettingsCard>
          <div className="ctd-section">
            <div className="ctd-section-label">{t("connectors.detail.description")}</div>
            <div className="ctd-section-value">{getMcpDescription(connector, i18n.language)}</div>
          </div>
        </SettingsCard>

        <SettingsCard>
          <div className="ctd-section">
            <div className="ctd-section-label">{t("connectors.detail.tools")}</div>
            <div className="ctd-tools">
              {connector.tools.map((tool) => (
                <span key={tool} className="ctd-tool-badge">{tool}</span>
              ))}
            </div>
          </div>
        </SettingsCard>

        <SettingsCard>
          <div className="ctd-row ctd-row-border">
            <span className="ctd-row-label">{t("connectors.detail.author")}</span>
            <span className="ctd-row-value">{connector.author}</span>
          </div>
          <div className="ctd-row">
            <span className="ctd-row-label">{t("connectors.detail.website")}</span>
            <button
              type="button"
              className="ctd-link"
              onClick={() => void open(connector.url)}
            >
              {t("connectors.detail.openSite")} <ArrowSquareOut size="var(--icon-xs)" />
            </button>
          </div>
        </SettingsCard>

        {confirmDelete && (
          <button type="button" className="ak-confirm-delete" onClick={() => void onDelete().then(() => setConfirmDelete(false))}>
            {t("connectors.detail.confirmDeleteBtn")}
          </button>
        )}
      </div>
    </div>
  );
}
