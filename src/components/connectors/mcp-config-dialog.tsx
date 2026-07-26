import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { X } from "@/components/ui/icons";
import { McpIcon } from "@/lib/mcp-icons";
import { getMcpDescription } from "@/types/mcp";
import type { McpConnectorSpec } from "@/types/mcp";
import "./mcp-config-dialog.css";

type TestState = { kind: "idle" } | { kind: "testing" } | { kind: "ok" } | { kind: "error"; message: string };

interface McpConfigDialogProps {
  connector: McpConnectorSpec;
  onClose: () => void;
  onValidated: () => Promise<void>;
}

export function McpConfigDialog({ connector, onClose, onValidated }: McpConfigDialogProps) {
  const { t, i18n } = useTranslation();
  const [token, setToken] = useState("");
  const [testState, setTestState] = useState<TestState>({ kind: "idle" });
  const [submitting, setSubmitting] = useState(false);

  const envKey = connector.env_keys?.[0];
  const storedConnector = {
    id: connector.id,
    status: "connected",
    enabled_in_chat: true,
    endpoint: connector.endpoint,
    install_command: connector.install_command,
    env_keys: connector.env_keys,
  };

  const handleClose = () => {
    setToken("");
    onClose();
  };

  const handleSubmit = async () => {
    if (!token.trim() || submitting || !envKey) return;
    setSubmitting(true);
    setTestState({ kind: "testing" });
    try {
      await invoke("configure_mcp_connector_tokens", {
        connector: storedConnector,
        envTokens: [{ env_key: envKey, value: token }],
      });
      setTestState({ kind: "ok" });
      setTimeout(() => {
        void onValidated().catch(() => {
          setTestState({ kind: "error", message: t("connectors.config.testError") });
          setSubmitting(false);
        });
      }, 500);
    } catch {
      setTestState({ kind: "error", message: t("connectors.config.testError") });
      setSubmitting(false);
    } finally {
      setToken("");
    }
  };

  return (
    <div className="wk-dialog-overlay" role="presentation" onClick={handleClose} onKeyDown={() => {}}>
      <div className="wk-dialog" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={() => {}}>
        <div className="wk-dialog-header">
          <h3>{t("connectors.config.addTitle", { name: connector.display_name })}</h3>
          <button type="button" className="wk-dialog-close" onClick={handleClose}><X size="var(--icon-md)" /></button>
        </div>

        <div className="mcc-info">
          <McpIcon connectorId={connector.id} displayName={connector.display_name} size={32} />
          <span className="mcc-desc">{getMcpDescription(connector, i18n.language)}</span>
        </div>
        <p className="mcc-security-warning">{t("connectors.localSecurityWarning")}</p>

        <div className="wk-form">
          <label className="wk-form-label">{t("connectors.config.token")}</label>
          <input
            type="password"
            className="wk-input"
            value={token}
            onChange={(e) => setToken(e.target.value)}
            placeholder={t("connectors.config.tokenPlaceholder")}
            autoFocus
          />
        </div>

        {testState.kind === "testing" && <div className="ak-test-result loading">{t("connectors.config.testing")}</div>}
        {testState.kind === "ok" && <div className="ak-test-result success">{t("connectors.config.testOk")}</div>}
        {testState.kind === "error" && <div className="ak-test-result error">{testState.message}</div>}

        <div className="wk-dialog-footer">
          <button type="button" className="wk-btn-secondary" onClick={handleClose}>{t("connectors.config.cancel")}</button>
          <button type="button" className="wk-btn-primary" onClick={() => void handleSubmit()} disabled={!token.trim() || submitting}>
            {t("connectors.config.addAndTest")}
          </button>
        </div>
      </div>
    </div>
  );
}
