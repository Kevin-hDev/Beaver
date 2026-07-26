import telegramIcon from "@/assets/channels/telegram.svg?url";
import discordIcon from "@/assets/channels/discord.svg?url";
import { Hash } from "@/components/ui/icons";
import type { ChannelType } from "@/types/channels";
import "./channel-icon.css";

interface ChannelIconProps {
  channelId: ChannelType;
  size?: number | string;
  className?: string;
}

const ICONS: Partial<Record<ChannelType, string>> = {
  telegram: telegramIcon,
  discord: discordIcon,
};

export function ChannelIcon({ channelId, size = 18, className = "" }: ChannelIconProps) {
  const cls = className ? `channel-icon ${className}` : "channel-icon";
  const src = ICONS[channelId];

  if (!src) {
    return <Hash size={size} weight="regular" className={cls} aria-hidden="true" />;
  }

  return (
    <span className={cls} style={{ width: size, height: size }} aria-hidden="true">
      <img className="channel-icon-img" src={src} alt="" draggable={false} />
    </span>
  );
}
