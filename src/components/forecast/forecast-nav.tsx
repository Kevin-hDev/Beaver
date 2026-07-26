import { useEffect, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown } from "@/components/ui/icons";
import type { ForecastSection } from "@/hooks/use-forecast-panel";
import { focusLocalListItem, useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";
import "./forecast-nav.css";

const NAV_ITEMS: { id: ForecastSection; i18nKey: string }[] = [
  { id: "view", i18nKey: "forecast.nav.mainView" },
  { id: "comparisons", i18nKey: "forecast.nav.comparisons" },
  { id: "history", i18nKey: "forecast.nav.history" },
];

interface ForecastNavProps {
  open: boolean;
  activeSection: ForecastSection;
  onToggle: () => void;
  onSelect: (section: ForecastSection) => void;
}

export function ForecastNav({ open, activeSection, onToggle, onSelect }: ForecastNavProps) {
  const { t } = useTranslation();
  const rootRef = useRef<HTMLDivElement | null>(null);
  const panelRef = useRef<HTMLDivElement | null>(null);
  const pendingFocusDirection = useRef<1 | -1>(1);
  const activeItem = NAV_ITEMS.find((item) => item.id === activeSection) ?? NAV_ITEMS[0];
  const navItems = useMemo<LocalListNavItem[]>(() => NAV_ITEMS.map((item) => ({
    id: item.id,
    onSelect: () => onSelect(item.id),
  })), [onSelect]);
  const nav = useLocalListNavigation({
    items: navItems,
    enabled: open,
    selectedId: activeSection,
    onEscape: onToggle,
  });

  useEffect(() => {
    if (open) focusLocalListItem(panelRef.current, pendingFocusDirection.current);
  }, [open]);

  useEffect(() => {
    if (!open) return;

    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) onToggle();
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onToggle();
    };

    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [open, onToggle]);

  return (
    <div ref={rootRef} className="fc-nav-root" data-keyboard-scope="local">
      <button
        className={`fc-nav-trigger ${open ? "is-open" : ""}`}
        onClick={onToggle}
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          event.stopPropagation();
          pendingFocusDirection.current = event.key === "ArrowDown" ? 1 : -1;
          if (open) focusLocalListItem(panelRef.current, pendingFocusDirection.current);
          else onToggle();
        }}
      >
        <span className="fc-nav-label">{t(activeItem.i18nKey)}</span>
        <ChevronDown size="var(--icon-sm)" className={`fc-nav-chevron ${open ? "is-open" : ""}`} />
      </button>
      <div
        ref={panelRef}
        className={`fc-nav-panel ${open ? "is-open" : ""}`}
        role="menu"
        tabIndex={-1}
        onKeyDown={nav.listProps.onKeyDown}
      >
        {NAV_ITEMS.map((item) => (
          <button
            key={item.id}
            ref={nav.getItemRef(item.id)}
            className={`fc-nav-item ${activeSection === item.id ? "fc-nav-active" : ""}`}
            data-local-nav-item="true"
            data-local-nav-active={nav.isActive(item.id) ? "true" : undefined}
            tabIndex={open && nav.isActive(item.id) ? 0 : -1}
            onFocus={() => nav.activate(item.id)}
            onMouseEnter={() => nav.activate(item.id)}
            onKeyDown={nav.listProps.onKeyDown}
            onClick={() => onSelect(item.id)}
          >
            <span
              className="fc-nav-dot"
              style={{
                background: activeSection === item.id ? "var(--fc-nav-active)" : "transparent",
              }}
            />
            {t(item.i18nKey)}
          </button>
        ))}
      </div>
    </div>
  );
}
