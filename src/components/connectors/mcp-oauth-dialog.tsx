import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { McpIcon } from "@/lib/mcp-icons";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { McpConnectorSpec } from "@/types/mcp";
import "./mcp-oauth-dialog.css";

type OAuthState = "loading" | "waiting" | "testing" | "success" | "error";

interface McpOauthDialogProps {
  connector: McpConnectorSpec;
  onClose: () => void;
  onConnected: () => void;
}

export function McpOauthDialog({ connector, onClose, onConnected }: McpOauthDialogProps) {
  const { t } = useTranslation();
  const [state, setState] = useState<OAuthState>("loading");
  const [error, setError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  const startFlow = useCallback(() => {
    setState("loading");
    setError(null);
    invoke("start_mcp_oauth", {
      connectorId: connector.id,
      endpoint: connector.endpoint ?? "",
    })
      .then(() => { if (mountedRef.current) setState("waiting"); })
      .catch(() => {
        if (!mountedRef.current) return;
        setState("error");
        setError(t("connectors.oauth.errorGeneric"));
      });
  }, [connector.id, connector.endpoint, t]);

  useEffect(() => {
    mountedRef.current = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- startFlow sets initial loading state synchronously
    startFlow();
    const unlisten = listen<unknown>("mcp-oauth-result", (e) => {
      if (!mountedRef.current) return;
      const p = e.payload as Record<string, unknown>;
      if (typeof p?.connector_id !== "string" || typeof p?.success !== "boolean") return;
      if (p.connector_id !== connector.id) return;
      if (p.success) {
        setState("success");
        setTimeout(() => onConnected(), 600);
      } else {
        setState("error");
        setError(t("connectors.oauth.errorGeneric"));
      }
    });
    return () => {
      mountedRef.current = false;
      cleanupTauriListener(unlisten);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- mount-only effect, startFlow/onConnected are stable
  }, []);

  const handleClose = () => {
    invoke("cancel_mcp_oauth", { connectorId: connector.id }).catch(() => {});
    onClose();
  };

  return (
    <div className="wk-dialog-overlay" role="presentation" onClick={handleClose} onKeyDown={() => {}}>
      <div className="wk-dialog mco-dialog" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={() => {}}>
        <div className="mco-icons">
          <div className="mco-icon-box">
            <McpIcon connectorId={connector.id} displayName={connector.display_name} size={40} />
          </div>
          <div className="mco-connector-line" />
          <div className="mco-icon-box">
            <span className="mco-app-icon">CL</span>
          </div>
        </div>

        <h3 className="mco-title">
          {state === "success" ? t("connectors.oauth.successTitle") : t("connectors.oauth.title")}
        </h3>

        {state === "loading" && <p className="mco-message">{t("connectors.oauth.discovering")}</p>}
        {state === "waiting" && <p className="mco-message">{t("connectors.oauth.message")}</p>}
        {state === "testing" && <p className="mco-message">{t("connectors.oauth.testing")}</p>}
        {state === "success" && <p className="mco-message">{t("connectors.oauth.successMessage")}</p>}
        {state === "error" && <p className="mco-message mco-error">{error}</p>}

        {(state === "waiting" || state === "error") && (
          <button type="button" className="mco-retry-link" onClick={startFlow}>
            {t("connectors.oauth.retry")}
          </button>
        )}

        <div className="wk-dialog-footer mco-footer">
          <button type="button" className="wk-btn-secondary" onClick={handleClose}>
            {t("connectors.oauth.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
