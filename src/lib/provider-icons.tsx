import groqSvg from "@/assets/groq.svg";
import geminiPng from "@/assets/GEMINI.png";
import mistralSvg from "@/assets/mistral-color.svg";
import cerebrasSvg from "@/assets/cerebras-color.svg";
import openrouterSvg from "@/assets/openrouter.svg";
import openaiSvg from "@/assets/openai.svg";
import deepseekSvg from "@/assets/deepseek-color.svg";
import braveIconSvg from "@/assets/brave/Brave-icon.svg";
import exaSvg from "@/assets/exa-color.svg";
import firecrawlIconSvg from "@/assets/Firecrawl/Firecrawl-icon.svg";
import xaiSvg from "@/assets/Grok/grok-icon.svg";
import moonshotSvg from "@/assets/moonshot-ai/moonshot-icon.svg";
import zaiSvg from "@/assets/Z/Z.ai.svg";

interface ImgEntry { kind: "img"; src: string; mono?: boolean }

type ProviderIconEntry = ImgEntry;

const ICONS: Record<string, ProviderIconEntry> = {
  groq:       { kind: "img", src: groqSvg, mono: true },
  google:     { kind: "img", src: geminiPng },
  mistral:    { kind: "img", src: mistralSvg },
  cerebras:   { kind: "img", src: cerebrasSvg },
  openrouter: { kind: "img", src: openrouterSvg, mono: true },
  openai:     { kind: "img", src: openaiSvg, mono: true },
  deepseek:   { kind: "img", src: deepseekSvg },
  brave:      { kind: "img", src: braveIconSvg },
  exa:        { kind: "img", src: exaSvg },
  firecrawl:  { kind: "img", src: firecrawlIconSvg },
  xai:        { kind: "img", src: xaiSvg },
  moonshot:   { kind: "img", src: moonshotSvg },
  zai:        { kind: "img", src: zaiSvg },
};

interface ProviderIconProps {
  providerId: string;
  displayName: string;
  size?: number | string;
}

export function ProviderIcon({ providerId, displayName, size = 40 }: ProviderIconProps) {
  const entry = ICONS[providerId];

  if (!entry) {
    const color = colorFor(providerId);
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

  return (
    <img
      src={entry.src}
      alt={displayName}
      className={entry.mono ? "provider-icon-mono" : undefined}
      style={{ width: size, height: size, borderRadius: 8, objectFit: "contain", flexShrink: 0 }}
    />
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
