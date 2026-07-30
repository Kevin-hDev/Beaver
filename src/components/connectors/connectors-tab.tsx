import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useConnectors } from "@/hooks/use-connectors";
import { McpIcon } from "@/lib/mcp-icons";
import { SettingsPanel } from "@/components/settings/shell/settings-panel";
import { SettingsEntryList } from "@/components/settings/shell/settings-entry-list";
import type { McpConnectorSpec } from "@/types/mcp";
import type { ConnectorsTabProps, DialogState } from "./connectors-tab-types";
import { ConnectorsDetail } from "./connectors-detail";
import { ConnectorsConfirmDialogs } from "./connectors-confirm-dialogs";
import { McpBrowseModal } from "./mcp-browse-modal";
import { McpConfigDialog } from "./mcp-config-dialog";
import { McpOauthDialog } from "./mcp-oauth-dialog";
import "./connectors-tab.css";

export function useConnectorsTabContent({ navState, onNavChange, onNavReplace }: ConnectorsTabProps): React.ReactNode {
  const { t } = useTranslation();
  const {
    catalog,
    configured,
    configuredIds,
    loadError,
    addConnector,
    removeConnector,
    toggleStatus,
  } = useConnectors();
  const selectedId = navState.connectorId;
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });
  const [confirmAddBusy, setConfirmAddBusy] = useState(false);
  const [confirmAddError, setConfirmAddError] = useState(false);

  const selected = useMemo(
    () => selectedId ? configured.find((c) => c.id === selectedId) ?? null : null,
    [configured, selectedId],
  );

  const entries = useMemo(
    () => configured.map((connector) => ({
      id: connector.id,
      label: connector.display_name,
      icon: (
        <McpIcon
          connectorId={connector.id}
          displayName={connector.display_name}
          size="var(--icon-lg)"
        />
      ),
      offlineLabel: connector.status === "disconnected"
        ? t("connectors.detail.disconnected")
        : undefined,
    })),
    [configured, t],
  );

  const handlePick = useCallback((spec: McpConnectorSpec) => {
    setConfirmAddError(false);
    if (spec.auth_type === "none") {
      setDialog({ kind: "confirm-add", connector: spec, returnTo: "browse" });
    } else if (spec.auth_type === "oauth") {
      setDialog({ kind: "oauth-pending", connector: spec, returnTo: "browse" });
    } else {
      setDialog({ kind: "config", connector: spec, returnTo: "browse" });
    }
  }, []);

  const handleDelete = useCallback(async (connectorId: string) => {
    onNavReplace({ connectorId: null });
    await removeConnector(connectorId);
  }, [onNavReplace, removeConnector]);

  const handleDisconnect = useCallback(async () => {
    if (dialog.kind !== "confirm-disconnect") return;
    await toggleStatus(dialog.connectorId);
    setDialog({ kind: "none" });
  }, [dialog, toggleStatus]);

  const closeToReturn = useCallback((returnTo: "browse" | "none") => {
    setDialog(returnTo === "browse" ? { kind: "browse" } : { kind: "none" });
  }, []);

  const handleConfirmAdd = useCallback(async (connector: McpConnectorSpec) => {
    if (confirmAddBusy) return;
    setConfirmAddBusy(true);
    setConfirmAddError(false);
    try {
      await addConnector(connector.id);
      onNavChange({ connectorId: connector.id });
      setDialog({ kind: "none" });
    } catch {
      setConfirmAddError(true);
    } finally {
      setConfirmAddBusy(false);
    }
  }, [addConnector, confirmAddBusy, onNavChange]);

  const browseButton = useMemo(() => (
    <button type="button" className="ak-connectors-btn" onClick={() => setDialog({ kind: "browse" })}>
      <Plus size="var(--icon-sm)" weight="bold" />
      {t("connectors.main.browseBtn")}
    </button>
  ), [t]);

  const detail = useMemo(() => (
    <>
      {selected ? (
        <SettingsPanel>
          <ConnectorsDetail
            connector={selected}
            onBack={() => onNavReplace({ connectorId: null })}
            onToggleStatus={() => {
              if (selected.status === "connected") {
                setDialog({ kind: "confirm-disconnect", connectorId: selected.id });
              } else {
                void toggleStatus(selected.id);
              }
            }}
            onDelete={() => handleDelete(selected.id)}
          />
        </SettingsPanel>
      ) : (
        <SettingsPanel title={t("settings.tabs.connectors")} action={browseButton}>
          <p className="settings-panel-description">{t("connectors.main.subtitle")}</p>
          <SettingsEntryList
            entries={entries}
            emptyMessage={t(loadError ? "connectors.sidebar.loadError" : "connectors.sidebar.empty")}
            onSelect={(id) => onNavChange({ connectorId: id })}
          />
        </SettingsPanel>
      )}

      {dialog.kind === "browse" && (
        <McpBrowseModal catalog={catalog} configuredIds={configuredIds} onPick={handlePick} onClose={() => setDialog({ kind: "none" })} />
      )}
      {dialog.kind === "config" && (
        <McpConfigDialog
          connector={dialog.connector}
          onClose={() => {
            setDialog(dialog.returnTo === "browse" ? { kind: "browse" } : { kind: "none" });
          }}
          onValidated={() => {
            onNavChange({ connectorId: dialog.connector.id });
            setDialog({ kind: "none" });
            return Promise.resolve();
          }}
        />
      )}
      {dialog.kind === "oauth-pending" && (
        <McpOauthDialog
          connector={dialog.connector}
          onClose={() => {
            setDialog(dialog.returnTo === "browse" ? { kind: "browse" } : { kind: "none" });
          }}
          onConnected={() => {
            onNavChange({ connectorId: dialog.connector.id });
            setDialog({ kind: "none" });
          }}
        />
      )}
      <ConnectorsConfirmDialogs
        configured={configured}
        dialog={dialog}
        confirmAddBusy={confirmAddBusy}
        confirmAddError={confirmAddError}
        onConfirmAdd={(connector) => void handleConfirmAdd(connector)}
        onDisconnect={() => void handleDisconnect()}
        onCloseAdd={closeToReturn}
        onCloseDisconnect={() => setDialog({ kind: "none" })}
      />
    </>
  ), [
    browseButton,
    catalog,
    configured,
    configuredIds,
    confirmAddBusy,
    confirmAddError,
    closeToReturn,
    dialog,
    entries,
    handleConfirmAdd,
    handleDelete,
    handleDisconnect,
    handlePick,
    loadError,
    onNavChange,
    onNavReplace,
    selected,
    t,
    toggleStatus,
  ]);

  return detail;
}
