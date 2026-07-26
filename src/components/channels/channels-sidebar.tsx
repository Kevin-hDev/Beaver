import { useTranslation } from "react-i18next";
import { ChannelIcon } from "./channel-icon";
import type { ChannelType, ChannelHealthEntry, ChannelAccountConfig } from "@/types/channels";

interface ConfiguredAccount {
  channelId: ChannelType;
  accountId: string;
  config: ChannelAccountConfig;
}

interface ChannelsSidebarProps {
  accounts: ConfiguredAccount[];
  healthEntries: ChannelHealthEntry[];
  selectedKey: string | null;
  onSelect: (key: string) => void;
}

export function ChannelsSidebar({ accounts, healthEntries, selectedKey, onSelect }: ChannelsSidebarProps) {
  const { t } = useTranslation();

  const statusFor = (channelId: string, accountId: string) =>
    healthEntries.find((e) => e.channel_id === channelId && e.account_id === accountId)?.status ?? "off";

  return (
    <div className="cts-sidebar">
      <div className="cts-header">{t("channels.sidebar.configured")}</div>
      <div className="cts-list">
        {accounts.length === 0 ? (
          <div className="cts-empty">{t("channels.sidebar.empty")}</div>
        ) : (
          accounts.map((acc) => {
            const key = `${acc.channelId}:${acc.accountId}`;
            const status = statusFor(acc.channelId, acc.accountId);
            return (
              <button
                key={key}
                type="button"
                className={`cts-item ${selectedKey === key ? "active" : ""}`}
                onClick={() => onSelect(key)}
              >
                <ChannelIcon channelId={acc.channelId} size="var(--icon-lg)" />
                <span>{acc.accountId}</span>
                {status !== "running" && <span className="cts-dot-off" />}
              </button>
            );
          })
        )}
      </div>
    </div>
  );
}
