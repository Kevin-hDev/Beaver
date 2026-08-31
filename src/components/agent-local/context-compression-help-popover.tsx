import { useCallback, useEffect, useId, useRef, useState, type CSSProperties } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import badgeQuestionMark from "@/assets/lucide--badge-question-mark.svg";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
import "./context-compression-help-popover.css";

interface ContextCompressionHelpPopoverProps {
  onOpenChange: (open: boolean) => void;
}

export function ContextCompressionHelpPopover({
  onOpenChange,
}: ContextCompressionHelpPopoverProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement | null>(null);
  const panelRef = useRef<HTMLElement | null>(null);
  const popoverId = useId();
  const titleId = useId();
  const { anchorRef, floatingRef, floatingStyle } = useFloatingMenuPosition(
    open,
    "before",
    8,
    "auto",
    false,
    undefined,
    panelRef,
  );

  const changeOpen = useCallback((next: boolean) => {
    setOpen(next);
    onOpenChange(next);
  }, [onOpenChange]);

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      changeOpen(false);
      buttonRef.current?.focus();
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => {
      document.removeEventListener("keydown", onKeyDown, true);
    };
  }, [changeOpen, floatingRef, open]);

  const setButton = (node: HTMLButtonElement | null) => {
    buttonRef.current = node;
    anchorRef.current = node;
    panelRef.current = node?.closest<HTMLElement>(".context-ring-panel") ?? null;
  };

  return (
    <>
      <button
        ref={setButton}
        type="button"
        className="cch-trigger"
        aria-label={t("agentLocal.contextUsage.compressionHelpTitle")}
        aria-expanded={open}
        aria-controls={popoverId}
        onClick={() => changeOpen(!open)}
      >
        <span
          className="cch-icon"
          style={{ "--cch-icon": `url(${badgeQuestionMark})` } as CSSProperties}
          aria-hidden="true"
        />
      </button>
      {open && createPortal(<>
        <button
          type="button"
          className="cch-shield"
          tabIndex={-1}
          aria-label={t("agentLocal.contextUsage.compressionHelpDismiss")}
          onClick={() => {
            changeOpen(false);
            buttonRef.current?.focus();
          }}
        />
        <div
          ref={floatingRef}
          id={popoverId}
          className="cch-popover"
          style={{ ...floatingStyle, zIndex: "var(--z-dialog)" }}
          role="dialog"
          aria-modal="false"
          aria-labelledby={titleId}
        >
          <strong id={titleId}>{t("agentLocal.contextUsage.compressionHelpTitle")}</strong>
          <p>{t("agentLocal.contextUsage.compressionHelp")}</p>
        </div>
      </>,
        floatingMenuPortalRoot(),
      )}
    </>
  );
}
