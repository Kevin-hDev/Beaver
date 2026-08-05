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
          {/* Cadre de 8 pour un glyphe de 4 : les bouts arrondis du trait dépassent
              de la moitié de son épaisseur, et un cadre serré les rognerait. */}
          <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
            <line x1="2" y1="2" x2="6" y2="6" stroke="currentColor" strokeWidth="1.25" strokeLinecap="round"/>
            <line x1="6" y1="2" x2="2" y2="6" stroke="currentColor" strokeWidth="1.25" strokeLinecap="round"/>
          </svg>
        </span>
      </button>
      <button className="wc-btn wc-btn--minimize" onClick={handleMinimize} tabIndex={-1} aria-label={t("a11y.minimize")}>
        <span className="wc-icon" aria-hidden="true">
          <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
            <line x1="1.7" y1="4" x2="6.3" y2="4" stroke="currentColor" strokeWidth="1.25" strokeLinecap="round"/>
          </svg>
        </span>
      </button>
      <button className="wc-btn wc-btn--maximize" onClick={handleMaximize} tabIndex={-1} aria-label={t("a11y.maximize")}>
        <span className="wc-icon" aria-hidden="true">
          {/* Deux triangles pleins plutôt que deux équerres en fil : à cette taille
              les traits d'une équerre se rejoignent et forment une tache. */}
          <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
            <path d="M1.7 1.7h3.4L1.7 5.1z" fill="currentColor"/>
            <path d="M6.3 6.3H2.9L6.3 2.9z" fill="currentColor"/>
          </svg>
        </span>
      </button>
    </div>
  );
}
