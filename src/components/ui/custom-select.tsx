import { useCallback, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useClickOutside } from "@/hooks/use-click-outside";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
import { useLocalListNavigation } from "@/hooks/use-local-list-navigation";
import "./custom-select.css";

interface SelectOption {
  value: string;
  label: string;
}

interface CustomSelectProps {
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  ariaLabel?: string;
}

export function CustomSelect({
  options,
  value,
  onChange,
  placeholder,
  disabled,
  ariaLabel,
}: CustomSelectProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLElement | null>(null);
  const close = useCallback(() => setOpen(false), []);
  /* La liste est portée hors de son conteneur : un fondu, un flou ou un
     débordement masqué posé sur un ancêtre la coupait au bord de la carte et
     interdisait son propre flou. Elle reste sous le bouton et à sa largeur,
     comme le faisaient top/left/right. */
  const { anchorRef, floatingRef, floatingStyle } =
    useFloatingMenuPosition(open, "left", 4, "below", false, triggerRef);
  useClickOutside(ref, close, floatingRef);

  const selected = options.find((o) => o.value === value);
  const navItems = useMemo(() => options.map((option) => ({
    id: optionNavId(option.value),
    onSelect: () => {
      onChange(option.value);
      setOpen(false);
    },
  })), [onChange, options]);
  const { activate, getItemRef, isActive, listProps } = useLocalListNavigation({
    items: navItems,
    enabled: open && !disabled,
    selectedId: optionNavId(value),
    onEscape: close,
  });

  const dropdown = open ? (
    <div
      ref={floatingRef}
      style={floatingStyle}
      className="cs-dropdown"
      data-keyboard-scope="local"
    >
      <div className="cs-menu" role="listbox" tabIndex={-1} onKeyDown={listProps.onKeyDown}>
        {options.map((opt) => {
          const navId = optionNavId(opt.value);
          return (
          <div
            key={opt.value}
            className={`menu-row cs-option ${opt.value === value ? "active" : ""}`}
            role="option"
            ref={getItemRef(navId)}
            tabIndex={isActive(navId) ? 0 : -1}
            data-local-nav-item="true"
            data-local-nav-active={isActive(navId) ? "true" : undefined}
            aria-selected={opt.value === value}
            onFocus={() => activate(navId)}
            onMouseEnter={() => activate(navId)}
            onKeyDown={listProps.onKeyDown}
            onClick={() => {
              onChange(opt.value);
              setOpen(false);
            }}
          >
            {opt.label}
          </div>
        ); })}
      </div>
    </div>
  ) : null;

  return (
    <div ref={ref} className="cs-wrapper" data-keyboard-scope={open ? "local" : undefined}>
      <button
        type="button"
        ref={(node) => {
          anchorRef.current = node;
          triggerRef.current = node;
        }}
        className="btn btn-sm btn-secondary btn-select cs-trigger"
        onClick={() => !disabled && setOpen(!open)}
        onKeyDown={(event) => {
          if (disabled) return;
          if (!open && (event.key === "Enter" || event.key === " " || event.key === "ArrowDown")) {
            setOpen(true);
            return;
          }
          if (open) listProps.onKeyDown(event);
        }}
        disabled={disabled}
        aria-label={ariaLabel}
      >
        <span className={selected ? undefined : "cs-placeholder"}>
          {selected?.label ?? placeholder ?? "—"}
        </span>
        <span className="cs-trigger-caret">▾</span>
      </button>
      {dropdown ? createPortal(dropdown, floatingMenuPortalRoot()) : null}
    </div>
  );
}

function optionNavId(value: string) {
  return `option:${value}`;
}
