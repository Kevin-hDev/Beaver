import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useApiKeys } from "@/hooks/use-api-keys";
import { ProviderIcon } from "@/lib/provider-icons";
import { SettingsPanel } from "@/components/settings/shell/settings-panel";
import { SettingsEntryList } from "@/components/settings/shell/settings-entry-list";
import { ProvidersShell } from "@/components/providers/providers-shell";
import type { ProviderSpec } from "@/types/api";
import { ApiKeysDetails } from "./api-keys-details";
import { ApiKeysConfigDialog } from "./api-keys-config-dialog";
import { ConnectorsModal } from "./connectors-modal";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
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

export function useApiKeysTabContent({ navState, onNavChange, onNavReplace }: ApiKeysTabProps): React.ReactNode {
  const { t } = useTranslation();
  const { catalog, configuredIds, configured, setKey, deleteKey, testKeyRaw } =
    useApiKeys();
  const selectedId = navState.apiKeyProviderId;
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });

  const selected = useMemo(
    () => selectedId ? configured.find((p) => p.id === selectedId) ?? null : null,
    [configured, selectedId],
  );

  const entries = useMemo(
    () => configured.map((provider) => ({
      id: provider.id,
      label: provider.display_name,
      icon: (
        <ProviderIcon
          providerId={provider.id}
          displayName={provider.display_name}
          size="var(--icon-lg)"
        />
      ),
    })),
    [configured],
  );

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

  const connectorsButton = useMemo(() => (
    <button type="button" className="ak-connectors-btn" onClick={() => setDialog({ kind: "connectors" })}>
      <Plus size="var(--icon-sm)" weight="bold" />
      {t("apiKeys.main.connectorsBtn")}
    </button>
  ), [t]);

  const detail = useMemo(() => (
    <>
      {selected ? (
        <SettingsPanel>
          <ApiKeysDetails
            key={selected.id}
            provider={selected}
            onBack={() => onNavReplace({ apiKeyProviderId: null })}
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
        </SettingsPanel>
      ) : (
        <ProvidersShell
          active="api"
          action={connectorsButton}
          onChange={(providersSubTab) => onNavChange({ providersSubTab })}
        >
          <SettingsEntryList
            entries={entries}
            emptyMessage={t("apiKeys.empty.title")}
            onSelect={(id) => onNavChange({ apiKeyProviderId: id })}
          />
        </ProvidersShell>
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
    connectorsButton,
    deleteKey,
    dialog,
    entries,
    handleConfigClose,
    handleDelete,
    onNavChange,
    onNavReplace,
    selected,
    setKey,
    t,
    testKeyRaw,
  ]);

  return detail;
}
