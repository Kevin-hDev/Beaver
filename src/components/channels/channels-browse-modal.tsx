import { useEffect } from "react";
import type { MouseEvent } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { X, Plus, ArrowSquareOut } from "@/components/ui/icons";
import { ChannelIcon } from "./channel-icon";
import type { ChannelType } from "@/types/channels";

interface ChannelsBrowseModalProps {
  onPick: (channelId: ChannelType) => void;
  onClose: () => void;
}

interface ChannelSpec {
  id: ChannelType;
  name: string;
  descKey: string;
  category: string;
  url: string;
}

const CHANNEL_SPECS: ChannelSpec[] = [
  { id: "telegram", name: "Telegram", descKey: "channels.browse.telegramDesc", category: "messaging", url: "https://t.me/BotFather" },
  { id: "slack", name: "Slack", descKey: "channels.browse.slackDesc", category: "pro", url: "https://api.slack.com/apps" },
  { id: "discord", name: "Discord", descKey: "channels.browse.discordDesc", category: "pro", url: "https://discord.com/developers/applications" },
];

export function ChannelsBrowseModal({ onPick, onClose }: ChannelsBrowseModalProps) {
  const { t } = useTranslation();

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="wk-dialog-overlay" role="presentation" onClick={onClose} onKeyDown={() => {}}>
      <div className="ak-connectors-modal" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={() => {}}>
        <header className="ak-connectors-header">
          <div>
            <h2 style={{ margin: 0, fontSize: "var(--text-xl)", fontWeight: 700 }}>
              {t("channels.browse.title")}
            </h2>
            <p style={{ margin: "4px 0 0", fontSize: "var(--text-sm)", color: "var(--ink-muted)" }}>
              {t("channels.browse.subtitle")}
            </p>
          </div>
          <button type="button" className="wk-dialog-close" onClick={onClose}>
            <X size="var(--icon-md)" />
          </button>
        </header>

        <div className="ak-connectors-grid">
          {CHANNEL_SPECS.map((spec) => {
            const handleLinkClick = (e: MouseEvent) => {
              e.stopPropagation();
              void open(spec.url);
            };
            return (
              <div
                key={spec.id}
                className="ak-connector-card"
                role="button"
                tabIndex={0}
                onClick={() => onPick(spec.id)}
                onKeyDown={(e) => { if (e.key === "Enter") onPick(spec.id); }}
              >
                <div className="ch-browse-icon">
                  <ChannelIcon channelId={spec.id} size={32} />
                </div>
                <div className="ak-connector-card-body">
                  <div className="ak-connector-card-name">{spec.name}</div>
                  <div className="ak-connector-card-desc">{t(spec.descKey)}</div>
                  <div className="ak-connector-card-meta">
                    <span className="mcbc-cat">{t(`channels.browse.${spec.category}`)}</span>
                    <button type="button" className="mcbc-link" onClick={handleLinkClick} title={spec.url}>
                      <ArrowSquareOut size="var(--icon-xs)" />
                    </button>
                  </div>
                </div>
                <div className="ak-connector-card-action">
                  <Plus size="var(--icon-md)" weight="bold" />
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
