import { useTranslation } from "react-i18next";
import type { ConfiguredMcpFull, McpConnectorSpec } from "@/types/mcp";
import type { DialogState } from "./connectors-tab-types";

interface ConnectorsConfirmDialogsProps {
  configured: ConfiguredMcpFull[];
  dialog: DialogState;
  confirmAddBusy: boolean;
  confirmAddError: boolean;
  onConfirmAdd: (connector: McpConnectorSpec) => void;
  onDisconnect: () => void;
  onCloseAdd: (returnTo: "browse" | "none") => void;
  onCloseDisconnect: () => void;
}

export function ConnectorsConfirmDialogs({
  configured,
  dialog,
  confirmAddBusy,
  confirmAddError,
  onConfirmAdd,
  onDisconnect,
  onCloseAdd,
  onCloseDisconnect,
}: ConnectorsConfirmDialogsProps) {
  const { t } = useTranslation();

  if (dialog.kind === "confirm-add") {
    return (
      <div className="wk-dialog-overlay" role="presentation" onClick={() => onCloseAdd(dialog.returnTo)} onKeyDown={() => {}}>
        <div className="wk-dialog" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={() => {}}>
          <h3>{t("connectors.config.addTitle", { name: dialog.connector.display_name })}</h3>
          <p className="ct-confirm-desc">{t("connectors.config.confirmAddDesc", { name: dialog.connector.display_name })}</p>
          {dialog.connector.install_command && (
            <p className="mcc-security-warning">{t("connectors.localSecurityWarning")}</p>
          )}
          {confirmAddError && <div className="ak-test-result error">{t("connectors.config.testError")}</div>}
          <div className="wk-dialog-footer">
            <button type="button" className="wk-btn-secondary" onClick={() => onCloseAdd(dialog.returnTo)}>{t("connectors.detail.cancel")}</button>
            <button type="button" className="wk-btn-primary" disabled={confirmAddBusy} onClick={() => onConfirmAdd(dialog.connector)}>{t("connectors.config.confirmAddBtn")}</button>
          </div>
        </div>
      </div>
    );
  }

  if (dialog.kind !== "confirm-disconnect") return null;

  const connectorName = configured.find((c) => c.id === dialog.connectorId)?.display_name;
  return (
    <div className="wk-dialog-overlay" role="presentation" onClick={onCloseDisconnect} onKeyDown={() => {}}>
      <div className="wk-dialog" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={() => {}}>
        <h3>{t("connectors.detail.confirmDisconnectTitle", { name: connectorName })}</h3>
        <p className="ct-confirm-desc">{t("connectors.detail.confirmDisconnectDesc", { name: connectorName })}</p>
        <div className="wk-dialog-footer">
          <button type="button" className="wk-btn-secondary" onClick={onCloseDisconnect}>{t("connectors.detail.cancel")}</button>
          <button type="button" className="wk-btn-primary ct-btn-danger" onClick={onDisconnect}>{t("connectors.detail.confirmDisconnectBtn")}</button>
        </div>
      </div>
    </div>
  );
}
