import "./context-progress.css";
import { useCallback, useEffect, useId, useRef, useState } from "react";
import { AppSurfacePortal } from "@/components/ui/app-surface-portal";
import { useTranslation } from "react-i18next";
import type { ContextUsageBreakdown } from "@/hooks/context-usage-breakdown";
import type { ResolvedCompressionProfileView } from "@/types/compression-profile.generated";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
import { useContextProgressSurfaceEffects } from "./use-context-progress-surface-effects";
import { useAppSurfaceActive } from "@/components/layout/app-surface-activity";
import type { ResolvedContextUsage } from "@/hooks/agent-token-estimate";
import { ContextProgressPanel } from "./context-progress-panel";

interface ContextProgressProps {
  breakdown: ContextUsageBreakdown;
  compression?: ResolvedCompressionProfileView | null;
  summary?: ResolvedContextUsage;
}

type ColorKey = "neutral" | "yellow" | "orange" | "red";

function colorForPercentage(p: number): ColorKey {
  if (p >= 90) return "red";
  if (p >= 70) return "orange";
  if (p >= 55) return "yellow";
  return "neutral";
}

const FILL_COLORS: Record<ColorKey, string> = {
  neutral: "var(--context-ring-fill)",
  yellow: "var(--signal-warning)",
  orange: "var(--signal-alert)",
  red: "var(--signal-error)",
};

const SIZE = 16;
const STROKE = 3;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function ContextProgress({ breakdown, compression, summary }: ContextProgressProps) {
  const { t } = useTranslation();
  const surfaceActive = useAppSurfaceActive();
  const [open, setOpen] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);
  const hostRef = useRef<HTMLSpanElement | null>(null);
  const buttonRef = useRef<HTMLButtonElement | null>(null);
  const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const suppressNextFocusOpen = useRef(false);
  const focusPanelOnOpen = useRef(false);
  const panelId = useId();
  const { anchorRef, floatingRef, floatingStyle } = useFloatingMenuPosition(
    open,
    "left",
    8,
    "auto",
  );

  const cancelClose = () => {
    if (closeTimer.current) clearTimeout(closeTimer.current);
    closeTimer.current = null;
  };
  const openPanel = () => {
    cancelClose();
    setOpen(true);
  };
  const scheduleClose = () => {
    cancelClose();
    if (helpOpen) return;
    closeTimer.current = setTimeout(() => setOpen(false), 100);
  };
  const handleEscape = useCallback(() => {
    setOpen(false);
    suppressNextFocusOpen.current = true;
    buttonRef.current?.focus();
  }, []);
  const handleOutsideClick = useCallback(() => setOpen(false), []);

  useContextProgressSurfaceEffects({
    open,
    helpOpen,
    hostRef,
    floatingRef,
    focusPanelOnOpenRef: focusPanelOnOpen,
    onEscape: handleEscape,
    onOutsideClick: handleOutsideClick,
  });

  useEffect(() => () => cancelClose(), []);
  useEffect(() => {
    if (!surfaceActive) cancelClose();
  }, [surfaceActive]);

  if (!summary) return null;

  const used = breakdown.used;
  const percentage = used !== null && summary.max
    ? Math.min((used / summary.max) * 100, 100)
    : null;
  const colorKey = colorForPercentage(percentage ?? 0);
  const offset = CIRCUMFERENCE - ((percentage ?? 0) / 100) * CIRCUMFERENCE;

  const setHost = (node: HTMLSpanElement | null) => {
    hostRef.current = node;
    anchorRef.current = node;
  };
  const handleFocus = () => {
    if (suppressNextFocusOpen.current) {
      suppressNextFocusOpen.current = false;
      return;
    }
    openPanel();
  };

  return (
    <span
      ref={setHost}
      className="context-ring"
      onMouseEnter={openPanel}
      onMouseLeave={scheduleClose}
      onFocus={handleFocus}
      onBlur={scheduleClose}
    >
      <button
        ref={buttonRef}
        type="button"
        className="icon-btn context-ring-button"
        aria-label={t("agentLocal.contextUsage.title")}
        aria-expanded={open}
        aria-controls={panelId}
        onClick={openPanel}
        onKeyDown={(event) => {
          if (event.key !== "Enter" && event.key !== " ") return;
          event.preventDefault();
          focusPanelOnOpen.current = true;
          openPanel();
          if (open) requestAnimationFrame(() => floatingRef.current?.focus());
        }}
      >
        <svg width={SIZE} height={SIZE} viewBox={`0 0 ${SIZE} ${SIZE}`}>
          <circle
            className="context-ring-track"
            cx={SIZE / 2}
            cy={SIZE / 2}
            r={RADIUS}
            strokeWidth={STROKE}
          />
          <circle
            className="context-ring-fill"
            cx={SIZE / 2}
            cy={SIZE / 2}
            r={RADIUS}
            strokeWidth={STROKE}
            stroke={FILL_COLORS[colorKey]}
            strokeDasharray={CIRCUMFERENCE}
            strokeDashoffset={offset}
          />
        </svg>
      </button>
      {open && <AppSurfacePortal target={floatingMenuPortalRoot()}><div
        ref={floatingRef}
        id={panelId}
        className="context-ring-panel"
        style={floatingStyle}
        role="dialog"
        tabIndex={-1}
        aria-modal="false"
        aria-label={t("agentLocal.contextUsage.title")}
        onMouseEnter={openPanel}
        onMouseLeave={scheduleClose}
        onFocus={openPanel}
        onBlur={scheduleClose}
      >
        <ContextProgressPanel
          summary={{ ...summary, used }}
          breakdown={breakdown}
          compression={compression}
          percentage={percentage}
          onCompressionHelpOpen={(next) => {
            if (next) cancelClose();
            setHelpOpen(next);
          }}
        />
      </div></AppSurfacePortal>}
    </span>
  );
}
