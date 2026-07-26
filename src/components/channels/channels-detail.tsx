import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Trash } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { SettingsCard } from "@/components/settings/settings-card";
import { CustomSelect } from "@/components/ui/custom-select";
import { useAvailableModels, withoutInteractiveOnlyModels } from "@/hooks/use-available-models";
import { ChannelsAllowlist } from "./channels-allowlist";
import { ChannelIcon } from "./channel-icon";
import { channelErrorKey } from "./channel-error";
import type { ChannelType, ChannelHealthEntry, GatewayConfig, ChannelAccountConfig } from "@/types/channels";
import "./channels.css";

interface ChannelsDetailProps {
  channelId: ChannelType;
  account: ChannelAccountConfig;
  status: ChannelHealthEntry | undefined;
  config: GatewayConfig;
  onSaveConfig: (cfg: GatewayConfig) => Promise<void>;
  onDelete: () => void;
}

export function ChannelsDetail({ channelId, account, status, config, onSaveConfig, onDelete }: ChannelsDetailProps) {
  const { t } = useTranslation();
  const statusKey = status?.status ?? "off";
  const isRunning = statusKey === "running";
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [operationFailed, setOperationFailed] = useState(false);
  const { groups } = useAvailableModels();
  const channelGroups = useMemo(() => withoutInteractiveOnlyModels(groups), [groups]);

  const runOperation = async (operation: () => Promise<void>) => {
    setOperationFailed(false);
    try {
      await operation();
    } catch {
      setOperationFailed(true);
    }
  };

  useEffect(() => {
    if (!confirmDelete) return;
    const timer = setTimeout(() => setConfirmDelete(false), 5000);
    return () => clearTimeout(timer);
  }, [confirmDelete]);

  const updateAccount = async (patch: Partial<ChannelAccountConfig>) => {
    const list = [...(config.channels[channelId] ?? [])];
    const idx = list.findIndex((a) => a.account_id === account.account_id);
    if (idx < 0) return;
    list[idx] = { ...list[idx], ...patch };
    const updated = { ...config, channels: { ...config.channels, [channelId]: list } };
    await runOperation(() => onSaveConfig(updated));
  };

  const handleConnect = async () => {
    await runOperation(async () => {
      if (isRunning) {
        await invoke("gateway_stop");
        return;
      }
      const list = [...(config.channels[channelId] ?? [])];
      const idx = list.findIndex((a) => a.account_id === account.account_id);
      if (idx >= 0) list[idx] = { ...list[idx], enabled: true };
      const updated = { ...config, enabled: true, channels: { ...config.channels, [channelId]: list } };
      await onSaveConfig(updated);
      await invoke("gateway_start");
    });
  };

  const handleDelete = async () => {
    await runOperation(async () => {
      await invoke("gateway_delete_token", { channelId, accountId: account.account_id, tokenKind: null });
      const list = (config.channels[channelId] ?? []).filter((a) => a.account_id !== account.account_id);
      const updated = { ...config, channels: { ...config.channels, [channelId]: list } };
      await onSaveConfig(updated);
      onDelete();
    });
  };

  const providerOptions = Array.from(channelGroups.keys())
    .map((p) => ({ value: p, label: p.charAt(0).toUpperCase() + p.slice(1) }));

  const modelOptions = (channelGroups.get(account.provider) ?? [])
    .map((m) => ({ value: m.id, label: m.id }));

  const channelName = channelId.charAt(0).toUpperCase() + channelId.slice(1);

  return (
    <div className="ctd-scroll">
      <div className="ctd-inner">
        <div className="ctd-header">
          <div className="ctd-info">
            <ChannelIcon channelId={channelId} size={36} />
            <h2 className="ctd-name">{channelName} — {account.account_id}</h2>
          </div>
          <div className="ctd-actions">
            <button
              type="button"
              className={`ctd-status-btn ${isRunning ? "connected" : "disconnected"}`}
              onClick={() => void handleConnect()}
            >
              <span className="ctd-status-dot" />
              {t(isRunning ? "channels.detail.connected" : "channels.detail.disconnected")}
            </button>
            <Tooltip label={t("channels.detail.delete")} align="right">
              <button type="button" className="ak-icon-btn danger" onClick={() => setConfirmDelete(true)}>
                <Trash size="var(--icon-md)" />
              </button>
            </Tooltip>
          </div>
        </div>

        {status?.error && (
          <div className="ch-error-bubble">
            <span className="ch-error-text">{t(channelErrorKey(status.error))}</span>
          </div>
        )}
        {operationFailed && (
          <div className="ch-error-bubble"><span className="ch-error-text">{t("channels.errors.generic")}</span></div>
        )}

        <SettingsCard>
          <div className="ctd-section">
            <div className="ctd-section-label">{t("channels.config.description")}</div>
            <div className="ctd-section-value">{t(`channels.browse.${channelId}Desc`)}</div>
          </div>
        </SettingsCard>

        <SettingsCard>
          <div className="ctd-row ctd-row-border">
            <span className="ctd-row-label">
              {t(channelId === "slack" ? "channels.detail.botToken" : "channels.detail.token")}
            </span>
            <span className="ctd-row-value">••••••••</span>
          </div>
          {channelId === "slack" && (
            <div className="ctd-row ctd-row-border">
              <span className="ctd-row-label">{t("channels.detail.appToken")}</span>
              <span className="ctd-row-value">••••••••</span>
            </div>
          )}
          <div className="ctd-row ctd-row-border">
            <span className="ctd-row-label">{t("channels.status.off")}</span>
            <span className="ctd-row-value">{t(`channels.status.${statusKey}`)}</span>
          </div>
          <div className="ctd-row">
            <span className="ctd-row-label">{t("channels.detail.requireMention")}</span>
            <ToggleSwitch
              checked={account.require_mention}
              ariaLabel={t("channels.detail.requireMention")}
              onCheckedChange={(checked) => void updateAccount({ require_mention: checked })}
            />
          </div>
        </SettingsCard>

        <SettingsCard>
          <div className="ctd-section">
            <div className="ctd-section-label">{t("channels.detail.provider")}</div>
            <CustomSelect
              options={providerOptions}
              value={account.provider}
              onChange={(v) => void updateAccount({ provider: v, model: "" })}
              placeholder={t("channels.detail.provider")}
            />
          </div>
          <div className="ctd-section">
            <div className="ctd-section-label">{t("channels.detail.model")}</div>
            <CustomSelect
              options={modelOptions}
              value={account.model}
              onChange={(v) => void updateAccount({ model: v })}
              placeholder={t("channels.detail.model")}
              disabled={!account.provider}
            />
          </div>
        </SettingsCard>

        <SettingsCard>
          <div className="ctd-section">
            <div className="ctd-section-label">{t("channels.detail.allowlist")}</div>
            <ChannelsAllowlist
              allowlist={account.allowlist}
              onChange={(list) => void updateAccount({ allowlist: list })}
            />
          </div>
        </SettingsCard>

        {confirmDelete && (
          <button type="button" className="ak-confirm-delete" onClick={() => void handleDelete()}>
            {t("channels.detail.confirmDelete")}
          </button>
        )}
      </div>
    </div>
  );
}
