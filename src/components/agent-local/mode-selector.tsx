import { useState, useRef, useEffect, useCallback, useLayoutEffect } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import type { BrowserCapability } from "@/hooks/use-browser-capability";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import { resolveModeMenuPosition } from "./mode-selector-position";
import "./mode-selector.css";

interface ModeSelectorProps {
  mode: PanelMode;
  browserStatus?: BrowserCapability["status"];
  onChange: (mode: PanelMode) => void;
}

export function ModeSelector({ mode, browserStatus = "hidden", onChange }: ModeSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const btnRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ top: 0, right: 0 });

  const openMenu = useCallback(() => {
    if (btnRef.current) {
      const r = btnRef.current.getBoundingClientRect();
      setPos({ top: r.bottom, right: window.innerWidth - r.right });
    }
    setOpen(true);
  }, []);

  const closeMenu = useCallback(() => {
    setOpen(false);
  }, []);

  useLayoutEffect(() => {
    const anchor = btnRef.current;
    const menu = menuRef.current;
    if (!open || !anchor || !menu) return;
    const surface = document.querySelector<HTMLElement>('.ib-surface[data-native-active="true"]');
    const surfaceRect = surface?.getBoundingClientRect() ?? null;
    const next = resolveModeMenuPosition(
      anchor.getBoundingClientRect(),
      menu.getBoundingClientRect(),
      surfaceRect,
      { width: window.innerWidth, height: window.innerHeight },
    );
    setPos((current) => current.top === next.top && current.right === next.right ? current : next);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => {
      const target = e.target as Node;
      if (btnRef.current?.contains(target)) return;
      if (menuRef.current?.contains(target)) return;
      closeMenu();
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [closeMenu, open]);

  const pick = (m: PanelMode) => { onChange(m); closeMenu(); };

  return (
    <>
      <Tooltip label={t("forecast.panelMode.title")}>
        <button
          ref={btnRef}
          className={`icon-btn tab-action-btn ${mode !== "preview" ? "active" : ""}`}
          onClick={() => (open ? closeMenu() : openMenu())}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none"
            stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"
            strokeLinejoin="round">
            <rect x="1" y="2" width="6" height="12" rx="1.5" />
            <rect x="9" y="2" width="6" height="12" rx="1.5" />
            <path d="M5.5 8h5" strokeDasharray="2 2" />
          </svg>
        </button>
      </Tooltip>
      {open && createPortal(
        <div
          ref={menuRef}
          className="asp-mode-menu"
          style={{ top: pos.top, right: pos.right }}
        >
          <button className="asp-mode-item" onClick={() => pick("preview")}>
            <span className={`asp-mode-dot ${mode === "preview" ? "asp-mode-dot-active" : ""}`} />
            {t("forecast.panelMode.preview")}
          </button>
          <button className="asp-mode-item" onClick={() => pick("forecast")}>
            <span className={`asp-mode-dot ${mode === "forecast" ? "asp-mode-dot-active" : ""}`} />
            {t("forecast.panelMode.forecast")}
          </button>
          {browserStatus !== "hidden" && (
            <button
              className="asp-mode-item"
              disabled={browserStatus !== "ready"}
              title={browserStatus === "unavailable" ? t("browser.unavailable") : undefined}
              onClick={() => pick("browser")}
            >
              <span className={`asp-mode-dot ${mode === "browser" ? "asp-mode-dot-active" : ""}`} />
              {t("browser.title")}
            </button>
          )}
        </div>,
        document.body,
      )}
    </>
  );
}
