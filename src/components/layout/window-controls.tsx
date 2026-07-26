import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { IS_MAC } from "@/lib/platform";
import "./window-controls.css";

const DOUBLE_CLICK_MS = 300;

function handleClose() {
  getCurrentWindow().close().catch(() => {});
}

function handleMinimize() {
  getCurrentWindow().minimize().catch(() => {});
}

function handleMaximize() {
  const win = getCurrentWindow();
  win.isMaximized()
    .then((m) => (m ? win.unmaximize() : win.maximize()))
    .catch(() => {});
}

export function WindowControls() {
  const { t } = useTranslation();
  const lastClickRef = useRef(0);
  if (IS_MAC) return null;

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest(".wc-btn")) return;

    const now = Date.now();
    if (now - lastClickRef.current < DOUBLE_CLICK_MS) {
      lastClickRef.current = 0;
      handleMaximize();
      return;
    }
    lastClickRef.current = now;
    getCurrentWindow().startDragging().catch(() => {});
  };

  return (
    <div className="window-controls" role="presentation" onMouseDown={handleMouseDown}>
      <button className="wc-btn wc-btn--close" onClick={handleClose} tabIndex={-1} aria-label={t("a11y.close")}>
        <span className="wc-icon" aria-hidden="true">
          <svg width="6" height="6" viewBox="0 0 6 6" fill="none">
            <line x1="0.5" y1="0.5" x2="5.5" y2="5.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
            <line x1="5.5" y1="0.5" x2="0.5" y2="5.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
          </svg>
        </span>
      </button>
      <button className="wc-btn wc-btn--minimize" onClick={handleMinimize} tabIndex={-1} aria-label={t("a11y.minimize")}>
        <span className="wc-icon" aria-hidden="true">
          <svg width="6" height="2" viewBox="0 0 6 2" fill="none">
            <line x1="0.5" y1="1" x2="5.5" y2="1" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
          </svg>
        </span>
      </button>
      <button className="wc-btn wc-btn--maximize" onClick={handleMaximize} tabIndex={-1} aria-label={t("a11y.maximize")}>
        <span className="wc-icon" aria-hidden="true">
          <svg width="6" height="6" viewBox="0 0 6 6" fill="none">
            <polyline points="2,0.5 5.5,0.5 5.5,4" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
            <polyline points="4,5.5 0.5,5.5 0.5,2" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
          </svg>
        </span>
      </button>
    </div>
  );
}
