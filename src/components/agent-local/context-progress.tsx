import "./context-progress.css";
import { useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import type { ContextUsageBreakdown, ContextUsageItem } from "@/hooks/context-usage-breakdown";
import type { ResolvedCompressionProfileView } from "@/types/compression-profile.generated";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
import { ContextCompressionHelpPopover } from "./context-compression-help-popover";

interface ContextProgressProps {
  used: number;
  max: number;
  breakdown?: ContextUsageBreakdown;
  compression?: ResolvedCompressionProfileView | null;
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
  orange: "var(--tool-bash)",
  red: "var(--signal-error)",
};

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

const SIZE = 16;
const STROKE = 3;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function ContextProgress({ used, max, breakdown, compression }: ContextProgressProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);
  const hostRef = useRef<HTMLSpanElement | null>(null);
  const buttonRef = useRef<HTMLButtonElement | null>(null);
  const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const suppressNextFocusOpen = useRef(false);
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

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || helpOpen) return;
      event.preventDefault();
      setOpen(false);
      suppressNextFocusOpen.current = true;
      buttonRef.current?.focus();
    };
    const onOutsideClick = (event: MouseEvent) => {
      if (helpOpen) return;
      const target = event.target as Node;
      if (hostRef.current?.contains(target) || floatingRef.current?.contains(target)) return;
      event.preventDefault();
      event.stopPropagation();
      setOpen(false);
    };
    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("click", onOutsideClick, true);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("click", onOutsideClick, true);
    };
  }, [floatingRef, helpOpen, open]);

  useEffect(() => () => cancelClose(), []);

  if (!max || max <= 0) return null;
  const resolvedUsed = breakdown?.used ?? used;
  const percentage = Math.min((resolvedUsed / max) * 100, 100);
  const colorKey = colorForPercentage(percentage);
  const offset = CIRCUMFERENCE - (percentage / 100) * CIRCUMFERENCE;
  const pctDisplay = percentage < 1 ? "0" : percentage.toFixed(1);
  const items = breakdown?.items ?? [];

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
      {open && createPortal(<div
        ref={floatingRef}
        id={panelId}
        className="context-ring-panel"
        style={{ ...floatingStyle, zIndex: "var(--z-overlay)" }}
        role="dialog"
        aria-modal="false"
        aria-label={t("agentLocal.contextUsage.title")}
        onMouseEnter={openPanel}
        onMouseLeave={scheduleClose}
        onFocus={openPanel}
        onBlur={scheduleClose}
      >
        <div className="context-ring-header">
          <span>{t("agentLocal.contextUsage.title")}</span>
          <strong>{formatTokens(resolvedUsed)} / {formatTokens(max)} ({pctDisplay}%)</strong>
        </div>
        <div className="context-ring-bar" aria-hidden="true">
          <div className="context-ring-bar-fill" style={{ width: `${percentage}%` }} />
        </div>
        <div className="context-ring-list">
          {items.map((item) => (
            <ContextUsageRow key={item.key} item={item} />
          ))}
        </div>
        {compression && (
          <div className="context-ring-compression-row">
            {compression.available ? (
              <>
                <span>{t("agentLocal.contextUsage.compression")}</span>
                <strong title={compression.name}>{compression.name}</strong>
              </>
            ) : (
              <>
                <span>{t("agentLocal.contextUsage.compressionDisabled")}</span>
                <ContextCompressionHelpPopover onOpenChange={(next) => {
                  if (next) cancelClose();
                  setHelpOpen(next);
                }} />
              </>
            )}
          </div>
        )}
      </div>, floatingMenuPortalRoot())}
    </span>
  );
}

function ContextUsageRow({ item }: { item: ContextUsageItem }) {
  const { t } = useTranslation();
  return (
    <div className="context-ring-row">
      <span className={`context-ring-dot context-ring-dot-${item.key}`} aria-hidden="true" />
      <span className="context-ring-label">{t(`agentLocal.contextUsage.categories.${item.key}`)}</span>
      <span className="context-ring-values">
        {formatTokens(item.tokens)}
        <span>{formatShare(item.percentage)}%</span>
      </span>
    </div>
  );
}

function formatShare(value: number): string {
  if (value > 0 && value < 0.1) return "<0.1";
  return value.toFixed(1);
}
