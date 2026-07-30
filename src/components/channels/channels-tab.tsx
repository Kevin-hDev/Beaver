import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useChannels } from "@/hooks/use-channels";
import { SettingsPanel } from "@/components/settings/shell/settings-panel";
import { SettingsEntryList } from "@/components/settings/shell/settings-entry-list";
import type { ChannelType } from "@/types/channels";
import { ChannelIcon } from "./channel-icon";
import { ChannelsDetail } from "./channels-detail";
import { ChannelsBrowseModal } from "./channels-browse-modal";
import { ChannelsConfigDialog } from "./channels-config-dialog";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
import "./channels.css";

type DialogState =
  | { kind: "none" }
  | { kind: "browse" }
  | { kind: "config"; channelId: ChannelType; returnTo: "browse" | "none" };

interface ChannelsTabProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

export function useChannelsTabContent({ navState, onNavChange, onNavReplace }: ChannelsTabProps): React.ReactNode {
  const { t } = useTranslation();
  const { health, config, saveConfig, refreshHealth } = useChannels();
  const selectedKey = navState.channelKey;
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });

  const configuredAccounts = useMemo(() => {
    if (!config) return [];
    return (["telegram", "slack", "discord"] as ChannelType[]).flatMap((ch) =>
      (config.channels[ch] ?? []).map((acc) => ({ channelId: ch, accountId: acc.account_id, config: acc })),
    );
  }, [config]);

  const selected = useMemo(
    () => selectedKey
      ? configuredAccounts.find((a) => `${a.channelId}:${a.accountId}` === selectedKey) ?? null
      : null,
    [configuredAccounts, selectedKey],
  );

  const entries = useMemo(
    () => configuredAccounts.map((account) => {
      const status = health.channels.find(
        (entry) => entry.channel_id === account.channelId && entry.account_id === account.accountId,
      )?.status ?? "off";
      return {
        id: `${account.channelId}:${account.accountId}`,
        label: account.accountId,
        description: t(`channels.browse.${account.channelId}`),
        icon: <ChannelIcon channelId={account.channelId} size="var(--icon-lg)" />,
        offlineLabel: status === "running" ? undefined : t(`channels.status.${status}`),
      };
    }),
    [configuredAccounts, health.channels, t],
  );

  const handleConfigSaved = useCallback(async (channelId: ChannelType, accountId: string) => {
    if (!config) return;
    const list = [...(config.channels[channelId] ?? [])];
    if (!list.some((a) => a.account_id === accountId)) {
      const hasDefaultModel = Boolean(config.default_provider && config.default_model);
      list.push({
        account_id: accountId,
        enabled: hasDefaultModel,
        allowlist: [],
        require_mention: true,
        provider: config.default_provider,
        model: config.default_model,
      });
      await saveConfig({ ...config, channels: { ...config.channels, [channelId]: list } });
    }
    onNavChange({ channelKey: `${channelId}:${accountId}` });
    setDialog({ kind: "none" });
    await refreshHealth();
  }, [config, onNavChange, refreshHealth, saveConfig]);

  const browseButton = useMemo(() => (
    <button type="button" className="ak-connectors-btn" onClick={() => setDialog({ kind: "browse" })}>
      <Plus size="var(--icon-sm)" weight="bold" />
      {t("channels.main.browseBtn")}
    </button>
  ), [t]);

  const detail = useMemo(() => (
    <>
      {selected && config ? (
        <SettingsPanel>
          <ChannelsDetail
            channelId={selected.channelId}
            account={selected.config}
            status={health.channels.find((c) => c.channel_id === selected.channelId && c.account_id === selected.accountId)}
            config={config}
            onBack={() => onNavReplace({ channelKey: null })}
            onSaveConfig={saveConfig}
            onDelete={() => {
              onNavReplace({ channelKey: null });
              void refreshHealth();
            }}
          />
        </SettingsPanel>
      ) : (
        <SettingsPanel title={t("settings.tabs.channels")} action={browseButton}>
          <p className="settings-panel-description">{t("channels.main.subtitle")}</p>
          <SettingsEntryList
            entries={entries}
            emptyMessage={t("channels.sidebar.empty")}
            onSelect={(key) => onNavChange({ channelKey: key })}
          />
        </SettingsPanel>
      )}

      {dialog.kind === "browse" && (
        <ChannelsBrowseModal
          onPick={(channelId) => setDialog({ kind: "config", channelId, returnTo: "browse" })}
          onClose={() => setDialog({ kind: "none" })}
        />
      )}
      {dialog.kind === "config" && (
        <ChannelsConfigDialog
          channelId={dialog.channelId}
          onClose={() => setDialog(dialog.returnTo === "browse" ? { kind: "browse" } : { kind: "none" })}
          onSaved={(accountId: string) => void handleConfigSaved(dialog.channelId, accountId)}
        />
      )}
    </>
  ), [
    browseButton,
    config,
    dialog,
    entries,
    handleConfigSaved,
    health.channels,
    onNavChange,
    onNavReplace,
    refreshHealth,
    saveConfig,
    selected,
    t,
  ]);

  return detail;
}
