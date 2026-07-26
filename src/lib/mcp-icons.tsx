import apifyIconSvg from "@/assets/Apify-2/Apify-icon.svg?raw";
import apifyTextSvg from "@/assets/Apify-2/Apify-text.svg?raw";
import canvaIconSvg from "@/assets/Canva/canva-icon.svg?raw";
import canvaTextSvg from "@/assets/Canva/Canva-text.svg?raw";
import context7Png from "@/assets/context7/context7.png";
import figmaSvg from "@/assets/Figma/Figma.svg?raw";
import githubIconSvg from "@/assets/github/github.svg?raw";
import githubTextSvg from "@/assets/github/github-text.svg?raw";
import huggingfaceSvg from "@/assets/hugging-face/huggingface.svg?raw";
import imessageSvg from "@/assets/IMessage/IMessage_logo.svg?raw";
import linearIconSvg from "@/assets/Linear/Linear-icon.svg?raw";
import linearTextSvg from "@/assets/Linear/Linear-text.svg?raw";
import lucidPng from "@/assets/lucid/lucid.png";
import notionIconSvg from "@/assets/Notion/Notion-icon.svg?raw";
import notionTextSvg from "@/assets/Notion/Notion-text.svg?raw";
import producthuntSvg from "@/assets/Product_Hunt/Product-hunt.svg?raw";
import redditIconSvg from "@/assets/Reddit/Reddit-icon.svg?raw";
import redditTextSvg from "@/assets/Reddit/Reddit-text.svg?raw";
import sentryIconSvg from "@/assets/Sentry/Sentry-icon.svg?raw";
import sentryTextSvg from "@/assets/Sentry/Sentry-text.svg?raw";
import slackIconSvg from "@/assets/Slack-2/Slack-icon.svg?raw";
import slackTextSvg from "@/assets/Slack-2/Slack-text.svg?raw";
import vercelIconSvg from "@/assets/Vercel/Vercel-icon.svg?raw";
import vercelTextSvg from "@/assets/Vercel/Vercel-text.svg?raw";
import { useId, useMemo } from "react";
import { prepareMcpSvg } from "./mcp-svg-normalize";

type McpIconVariant = "icon" | "text";
type McpIconTone = "brand" | "mono";

interface SvgEntry { kind: "svg"; icon: string; text: string; prefix: string; tone: McpIconTone; hasText?: boolean }
interface ImgEntry { kind: "img"; src: string }

type McpIconEntry = SvgEntry | ImgEntry;

const MCP_ICONS: Record<string, McpIconEntry> = {
  apify:       { kind: "svg", tone: "brand", prefix: "apify-", icon: apifyIconSvg, text: apifyTextSvg, hasText: true },
  canva:       { kind: "svg", tone: "brand", prefix: "canva-", icon: canvaIconSvg, text: canvaTextSvg, hasText: true },
  context7:    { kind: "img", src: context7Png },
  figma:       { kind: "svg", tone: "brand", prefix: "figma-", icon: figmaSvg, text: figmaSvg },
  github:      { kind: "svg", tone: "mono", prefix: "gh-", icon: githubIconSvg, text: githubTextSvg, hasText: true },
  huggingface: { kind: "svg", tone: "brand", prefix: "hf-", icon: huggingfaceSvg, text: huggingfaceSvg },
  imessage:    { kind: "svg", tone: "brand", prefix: "im-", icon: imessageSvg, text: imessageSvg },
  linear:      { kind: "svg", tone: "mono", prefix: "lin-", icon: linearIconSvg, text: linearTextSvg, hasText: true },
  lucid:       { kind: "img", src: lucidPng },
  notion:      { kind: "svg", tone: "mono", prefix: "not-", icon: notionIconSvg, text: notionTextSvg, hasText: true },
  producthunt: { kind: "svg", tone: "brand", prefix: "ph-", icon: producthuntSvg, text: producthuntSvg },
  reddit:      { kind: "svg", tone: "brand", prefix: "red-", icon: redditIconSvg, text: redditTextSvg, hasText: true },
  sentry:      { kind: "svg", tone: "mono", prefix: "sen-", icon: sentryIconSvg, text: sentryTextSvg, hasText: true },
  slack:       { kind: "svg", tone: "brand", prefix: "slk-", icon: slackIconSvg, text: slackTextSvg, hasText: true },
  vercel:      { kind: "svg", tone: "mono", prefix: "ver-", icon: vercelIconSvg, text: vercelTextSvg, hasText: true },
};

export function mcpHasTextIcon(connectorId: string): boolean {
  const entry = MCP_ICONS[connectorId];
  return entry?.kind === "svg" && !!entry.hasText;
}

interface McpIconProps {
  connectorId: string;
  displayName: string;
  size?: number | string;
  variant?: McpIconVariant;
  textWidth?: boolean;
}

export function McpIcon({ connectorId, displayName, size = 40, variant = "icon", textWidth }: McpIconProps) {
  const entry = MCP_ICONS[connectorId];

  if (!entry) {
    return <McpIconFallback displayName={displayName} connectorId={connectorId} size={size} />;
  }

  if (entry.kind === "img") {
    return (
      <img
        src={entry.src}
        alt={displayName}
        style={{ width: size, height: size, borderRadius: 8, objectFit: "contain", flexShrink: 0 }}
      />
    );
  }

  return <McpSvgIcon entry={entry} size={size} variant={variant} textWidth={textWidth} />;
}

function McpSvgIcon({ entry, size, variant, textWidth }: {
  entry: SvgEntry;
  size: number | string;
  variant: McpIconVariant;
  textWidth?: boolean;
}) {
  const id = useId().replace(/[^a-zA-Z0-9_-]/g, "");
  const raw = variant === "text" ? entry.text : entry.icon;
  const svg = useMemo(() => prepareMcpSvg(raw, `${entry.prefix}${id}-`), [entry.prefix, id, raw]);
  const className = `mcp-icon-inline mcp-icon-${entry.tone}${textWidth ? " mcp-icon-text" : ""}`;

  if (textWidth) {
    return (
      <span
        className={className}
        style={{ height: size, display: "inline-flex", flexShrink: 0 }}
        dangerouslySetInnerHTML={{ __html: svg }}
      />
    );
  }

  return (
    <span
      className={className}
      style={{ width: size, height: size, display: "inline-flex", flexShrink: 0 }}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}

function McpIconFallback({ displayName, connectorId, size }: { displayName: string; connectorId: string; size: number | string }) {
  const color = colorFor(connectorId);
  return (
    <div style={{
      width: size, height: size, borderRadius: 8,
      background: `${color}22`, color,
      display: "flex", alignItems: "center", justifyContent: "center",
      fontWeight: 700,
      fontSize: typeof size === "number" ? size * 0.45 : `calc(${size} * 0.45)`,
      flexShrink: 0,
    }}>
      {displayName.charAt(0).toUpperCase()}
    </div>
  );
}

function colorFor(id: string): string {
  const palette = [
    "#f97316", "#3b82f6", "#10b981", "#8b5cf6", "#ec4899",
    "#eab308", "#06b6d4", "#ef4444", "#84cc16", "#a855f7",
  ];
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) >>> 0;
  }
  return palette[hash % palette.length];
}
