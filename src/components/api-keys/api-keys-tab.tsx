import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useApiKeys } from "@/hooks/use-api-keys";
import type { ProviderSpec } from "@/types/api";
import { ApiKeysSidebar } from "./api-keys-sidebar";
import { ApiKeysDetails } from "./api-keys-details";
import { ApiKeysConfigDialog } from "./api-keys-config-dialog";
import { ConnectorsModal } from "./connectors-modal";
import { EmptyState } from "@/components/ui/empty-state";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
import "./api-keys.css";
import "./api-keys-main.css";
import "./api-keys-detail.css";
import "./api-keys-dialog.css";
import "./connectors-modal.css";
import "./connector-card.css";

type DialogState =
  | { kind: "none" }
  | { kind: "connectors" }
  | {
      kind: "config";
      provider: ProviderSpec;
      alreadyConfigured: boolean;
      returnTo: "connectors" | "none";
    };

interface ApiKeysTabProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

export function useApiKeysTabSlots({ navState, onNavChange, onNavReplace }: ApiKeysTabProps): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const { catalog, configuredIds, configured, setKey, deleteKey, testKeyRaw } =
    useApiKeys();
  const selectedId = navState.apiKeyProviderId;
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });

  useEffect(() => {
    if (selectedId === null && configured.length > 0) {
      onNavReplace({ apiKeyProviderId: configured[0].id });
    }
  }, [selectedId, configured, onNavReplace]);

  const selected = useMemo(
    () => selectedId ? configured.find((p) => p.id === selectedId) ?? null : null,
    [configured, selectedId],
  );

  const list = useMemo(() => (
    <ApiKeysSidebar
      configured={configured}
      selectedId={selectedId}
      onSelect={(id) => onNavChange({ apiKeyProviderId: id })}
    />
  ), [configured, onNavChange, selectedId]);

  const handleDelete = useCallback(async () => {
    if (!selected) return;
    const id = selected.id;
    onNavReplace({ apiKeyProviderId: null });
    await deleteKey(id);
  }, [deleteKey, onNavReplace, selected]);

  const handleConfigClose = useCallback(() => {
    if (dialog.kind === "config" && dialog.returnTo === "connectors") {
      setDialog({ kind: "connectors" });
    } else {
      setDialog({ kind: "none" });
    }
  }, [dialog]);

  const detail = useMemo(() => (
    <>
      {selected ? (
        <ApiKeysDetails
          key={selected.id}
          provider={selected}
          onEdit={() =>
            setDialog({
              kind: "config",
              provider: selected,
              alreadyConfigured: true,
              returnTo: "none",
            })
          }
          onDelete={handleDelete}
          onAddConnector={() => setDialog({ kind: "connectors" })}
        />
      ) : (
        <div style={{ padding: 24, flex: 1, display: "flex", flexDirection: "column" }}>
          <div style={{ maxWidth: 600, width: "100%", margin: "0 auto" }}>
            <div style={{
              display: "flex", alignItems: "center",
              justifyContent: "space-between", marginBottom: 28,
            }}>
              <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", margin: 0 }}>
                {t("apiKeys.main.title")}
              </h2>
              <button type="button" className="ak-connectors-btn" onClick={() => setDialog({ kind: "connectors" })}>
                <Plus size="var(--icon-sm)" weight="bold" />
                {t("apiKeys.main.connectorsBtn")}
              </button>
            </div>
          </div>
          <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center" }}>
            <EmptyState message={t("apiKeys.empty.title")} />
          </div>
        </div>
      )}

      {dialog.kind === "connectors" && (
        <ConnectorsModal
          catalog={catalog}
          configuredIds={configuredIds}
          onPick={(p) =>
            setDialog({
              kind: "config",
              provider: p,
              alreadyConfigured: false,
              returnTo: "connectors",
            })
          }
          onClose={() => setDialog({ kind: "none" })}
        />
      )}

      {dialog.kind === "config" && (
        <ApiKeysConfigDialog
          provider={dialog.provider}
          alreadyConfigured={dialog.alreadyConfigured}
          onClose={handleConfigClose}
          onSave={async (key) => {
            await setKey(dialog.provider.id, key);
          }}
          onTest={async (key) => {
            await testKeyRaw(dialog.provider.id, key);
          }}
          onClearKey={
            dialog.alreadyConfigured
              ? async () => {
                  await deleteKey(dialog.provider.id);
                  onNavReplace({ apiKeyProviderId: null });
                }
              : undefined
          }
        />
      )}
    </>
  ), [
    catalog,
    configuredIds,
    deleteKey,
    dialog,
    handleConfigClose,
    handleDelete,
    onNavReplace,
    selected,
    setKey,
    t,
    testKeyRaw,
  ]);

  return useMemo(() => ({ list, detail }), [list, detail]);
}
