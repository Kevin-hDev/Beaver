import type { CSSProperties } from "react";
import { FolderOpen } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import {
  AGENT_SOURCE_LOGOS,
  type AgentLogoAsset,
} from "./agent-source-logo-assets";

interface AgentSourceLogoProps {
  sourceId: string;
  displayName: string;
  variant: "card" | "detail";
}

type LogoStyle = CSSProperties & {
  "--aim-logo-ratio"?: number;
  "--aim-logo-url"?: string;
};

function logoStyle(asset: AgentLogoAsset): LogoStyle {
  return {
    "--aim-logo-ratio": asset.ratio ?? 1,
    ...(asset.kind === "mono"
      ? { "--aim-logo-url": `url("${asset.src}")` }
      : {}),
  };
}

export function AgentSourceLogo({
  sourceId,
  displayName,
  variant,
}: AgentSourceLogoProps) {
  const asset = AGENT_SOURCE_LOGOS[sourceId]?.[variant];
  const isDetail = variant === "detail";
  const accessibilityProps = {
    "aria-hidden": isDetail ? undefined : true,
    "aria-label": isDetail ? displayName : undefined,
    role: isDetail ? "img" : undefined,
  };

  if (!asset) {
    return (
      <span
        className={cn("aim-source-logo", `aim-source-logo-${variant}`)}
        {...accessibilityProps}
      >
        <FolderOpen size="var(--icon-lg)" weight="duotone" />
      </span>
    );
  }

  return (
    <span
      className={cn("aim-source-logo", `aim-source-logo-${variant}`)}
      style={logoStyle(asset)}
      {...accessibilityProps}
    >
      {asset.kind === "color" && <img src={asset.src} alt="" />}
      {asset.kind === "mono" && <span className="aim-source-logo-mask" />}
      {asset.kind === "themed" && (
        <>
          <img
            className="aim-source-logo-themed themed-icon-light"
            src={asset.lightSrc}
            alt=""
          />
          <img
            className="aim-source-logo-themed themed-icon-dark"
            src={asset.darkSrc}
            alt=""
          />
        </>
      )}
    </span>
  );
}
