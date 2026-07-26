import { CaretDown } from "@/components/ui/icons";
import { createPortal } from "react-dom";
import { useEffect, useMemo, useRef, useState } from "react";
import { focusLocalListItem, useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
import "./forecast-scenario-menu.css";

interface ForecastScenarioMenuSelectProps {
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}

export function ForecastScenarioMenuSelect({
  value,
  options,
  onChange,
  placeholder,
  className,
}: ForecastScenarioMenuSelectProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const pendingFocusDirection = useRef<1 | -1>(1);
  const { anchorRef, floatingRef, floatingStyle } =
    useFloatingMenuPosition(open, "left", 6, "auto", true);
  const selected = options.find((option) => option.value === value);
  const navItems = useMemo<LocalListNavItem[]>(() => options.map((option) => ({
    id: option.value,
    onSelect: () => {
      onChange(option.value);
      setOpen(false);
    },
  })), [onChange, options]);
  const nav = useLocalListNavigation({
    items: navItems,
    enabled: open,
    selectedId: value,
    onEscape: () => setOpen(false),
  });

  useEffect(() => {
    if (!open) return;
    const handlePointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (
        !rootRef.current?.contains(target)
        && !floatingRef.current?.contains(target)
      ) {
        setOpen(false);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [floatingRef, open]);

  useEffect(() => {
    if (open) focusLocalListItem(floatingRef.current, pendingFocusDirection.current);
  }, [floatingRef, open]);

  const panel = open ? (
    <div
      ref={floatingRef}
      className="fcs-menu-panel"
      role="menu"
      tabIndex={-1}
      style={floatingStyle}
      data-keyboard-scope="local"
      onKeyDown={nav.listProps.onKeyDown}
    >
      {options.map((option) => (
        <button
          key={option.value}
          ref={nav.getItemRef(option.value)}
          className={`fcs-menu-option ${option.value === value ? "is-selected" : ""}`}
          type="button"
          data-local-nav-item="true"
          data-local-nav-active={nav.isActive(option.value) ? "true" : undefined}
          tabIndex={nav.isActive(option.value) ? 0 : -1}
          onFocus={() => nav.activate(option.value)}
          onMouseEnter={() => nav.activate(option.value)}
          onKeyDown={nav.listProps.onKeyDown}
          onClick={() => {
            onChange(option.value);
            setOpen(false);
          }}
        >
          <span className="fcs-menu-option-mark">
            {option.value === value ? <span className="fcs-menu-option-dot" /> : null}
          </span>
          <span>{option.label}</span>
        </button>
      ))}
    </div>
  ) : null;

  return (
    <div ref={rootRef} className={`fcs-menu ${className ?? ""}`.trim()} data-keyboard-scope="local">
      <button
        ref={(node) => { anchorRef.current = node; }}
        className={`btn btn-sm btn-secondary fcs-menu-trigger ${open ? "is-open" : ""}`}
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          event.stopPropagation();
          pendingFocusDirection.current = event.key === "ArrowDown" ? 1 : -1;
          if (open) focusLocalListItem(floatingRef.current, pendingFocusDirection.current);
          else setOpen(true);
        }}
      >
        <span className="fcs-menu-label">{selected?.label ?? placeholder ?? ""}</span>
        <CaretDown size="var(--icon-13)" className={`fcs-menu-caret ${open ? "is-open" : ""}`} />
      </button>
      {panel ? createPortal(panel, floatingMenuPortalRoot()) : null}
    </div>
  );
}
