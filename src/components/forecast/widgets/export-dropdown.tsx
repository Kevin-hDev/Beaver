import { useState, useRef, useEffect, useMemo } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { ClipboardText, DownloadSimple } from "@/components/ui/icons";
import { FileIcon } from "@/components/file-preview/file-icon";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
import { focusLocalListItem, useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";
import "./export-dropdown.css";

interface ExportDropdownProps {
  analysisId: string;
  onExport: (format: string, analysisId: string) => void | Promise<void>;
}

const FORMATS = [
  { key: "csv", fileName: "export.csv" },
  { key: "excel", fileName: "export.xlsx" },
  { key: "png", fileName: "export.png" },
  { key: "svg", fileName: "export.svg" },
  { key: "json", fileName: "export.json" },
  { key: "pdf", fileName: "export.pdf" },
  { key: "clipboard", fileName: null },
] as const;

export function ExportDropdown({ analysisId, onExport }: ExportDropdownProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const { anchorRef, floatingRef, floatingStyle } =
    useFloatingMenuPosition(open, "right", 6, "auto");
  const pendingFocusDirection = useRef<1 | -1>(1);
  const navItems = useMemo<LocalListNavItem[]>(() => FORMATS.map((format) => ({
    id: format.key,
    onSelect: () => {
      void onExport(format.key, analysisId);
      setOpen(false);
    },
  })), [analysisId, onExport]);
  const nav = useLocalListNavigation({
    items: navItems,
    enabled: open,
    selectedId: FORMATS[0].key,
    onEscape: () => setOpen(false),
  });

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => {
      const target = e.target as Node;
      if (ref.current?.contains(target) || floatingRef.current?.contains(target)) return;
      setOpen(false);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [floatingRef, open]);

  useEffect(() => {
    if (open) focusLocalListItem(floatingRef.current, pendingFocusDirection.current);
  }, [floatingRef, open]);

  const menu = open ? (
    <div
      ref={floatingRef}
      className="exd-menu"
      role="menu"
      tabIndex={-1}
      style={floatingStyle}
      onKeyDown={nav.listProps.onKeyDown}
    >
      {FORMATS.map((f) => (
        <button
          key={f.key}
          ref={nav.getItemRef(f.key)}
          className="exd-item"
          data-local-nav-item="true"
          data-local-nav-active={nav.isActive(f.key) ? "true" : undefined}
          tabIndex={nav.isActive(f.key) ? 0 : -1}
          onFocus={() => nav.activate(f.key)}
          onMouseEnter={() => nav.activate(f.key)}
          onKeyDown={nav.listProps.onKeyDown}
          onClick={() => {
            void onExport(f.key, analysisId);
            setOpen(false);
          }}
        >
          <span className="exd-icon">
            {f.fileName ? <FileIcon name={f.fileName} size="var(--icon-lg)" /> : <ClipboardText size="var(--icon-lg)" />}
          </span>
          {t(`forecast.export.${f.key}`)}
        </button>
      ))}
    </div>
  ) : null;

  return (
    <div className="exd-wrapper" ref={ref} data-keyboard-scope="local">
      <button
        ref={(node) => { anchorRef.current = node; }}
        className="exd-trigger"
        onClick={() => setOpen(!open)}
        aria-haspopup="menu"
        aria-expanded={open}
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          event.stopPropagation();
          pendingFocusDirection.current = event.key === "ArrowDown" ? 1 : -1;
          if (open) focusLocalListItem(floatingRef.current, pendingFocusDirection.current);
          else setOpen(true);
        }}
      >
        <DownloadSimple size="var(--icon-md)" />
        {t("forecast.export.title")}
      </button>
      {menu ? createPortal(menu, floatingMenuPortalRoot()) : null}
    </div>
  );
}
