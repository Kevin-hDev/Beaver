import { useCallback, useEffect, useLayoutEffect, useRef, useState, type CSSProperties } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { CaretDown, Check } from "@/components/ui/icons";
import { floatingMenuPortalRoot } from "@/hooks/use-floating-menu-position";
import type { FilePreviewListMode } from "@/types/file-preview";

interface FilePreviewSummarySelectorProps {
  active: boolean;
  mode: FilePreviewListMode;
  onSelect: () => void;
  onModeChange: (mode: FilePreviewListMode) => void;
}

const MODES: FilePreviewListMode[] = ["latest", "all", "uncommitted"];
const VIEWPORT_PADDING = 12;
const HIDDEN_STYLE: CSSProperties = {
  position: "fixed",
  top: 0,
  left: 0,
  visibility: "hidden",
  zIndex: 1000,
};

export function FilePreviewSummarySelector({
  active,
  mode,
  onSelect,
  onModeChange,
}: FilePreviewSummarySelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const [menuStyle, setMenuStyle] = useState<CSSProperties>(HIDDEN_STYLE);

  const updatePosition = useCallback(() => {
    const anchor = buttonRef.current;
    const menu = menuRef.current;
    if (!open || !anchor || !menu) return;

    const rect = anchor.getBoundingClientRect();
    const width = Math.max(190, rect.width);
    const height = menu.offsetHeight;
    const maxLeft = Math.max(VIEWPORT_PADDING, window.innerWidth - width - VIEWPORT_PADDING);
    const left = Math.min(Math.max(rect.left, VIEWPORT_PADDING), maxLeft);
    const below = rect.bottom + 6;
    const top = below + height <= window.innerHeight - VIEWPORT_PADDING
      ? below
      : Math.max(VIEWPORT_PADDING, rect.top - height - 6);

    setMenuStyle({
      position: "fixed",
      top,
      left,
      width,
      visibility: "visible",
      zIndex: 1000,
    });
  }, [open]);

  useLayoutEffect(() => {
    if (!open) return;
    updatePosition();
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);
    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [open, updatePosition]);

  useEffect(() => {
    if (!open) return;
    const closeOnPointer = (event: PointerEvent) => {
      const target = event.target as Node;
      if (buttonRef.current?.contains(target) || menuRef.current?.contains(target)) return;
      setOpen(false);
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", closeOnPointer);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("pointerdown", closeOnPointer);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [open]);

  const selectMode = (next: FilePreviewListMode) => {
    onModeChange(next);
    onSelect();
    setOpen(false);
  };

  return (
    <>
      <button
        ref={buttonRef}
        type="button"
        className={`fps-trigger ${active ? "active" : ""}`}
        onClick={() => {
          onSelect();
          setOpen((value) => !value);
        }}
        aria-haspopup="menu"
        aria-expanded={open}
      >
        <span className="fps-trigger-label">{t(`filePreview.listModes.${mode}`)}</span>
        <CaretDown size="var(--icon-2xs)" className="fps-trigger-caret" />
      </button>

      {open && createPortal(
        <div
          ref={menuRef}
          className="fps-menu"
          role="menu"
          tabIndex={-1}
          style={menuStyle}
        >
          {MODES.map((item) => (
            <button
              key={item}
              type="button"
              className="fps-menu-item"
              role="menuitemradio"
              aria-checked={item === mode}
              onClick={() => selectMode(item)}
            >
              <span>{t(`filePreview.listModes.${item}`)}</span>
              {item === mode && <Check size="var(--icon-sm)" />}
            </button>
          ))}
        </div>,
        floatingMenuPortalRoot(),
      )}
    </>
  );
}
