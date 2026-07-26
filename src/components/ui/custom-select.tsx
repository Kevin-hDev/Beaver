import { useCallback, useMemo, useRef, useState } from "react";
import { useClickOutside } from "@/hooks/use-click-outside";
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
  const close = useCallback(() => setOpen(false), []);
  useClickOutside(ref, close);

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

  return (
    <div ref={ref} className="cs-wrapper" data-keyboard-scope={open ? "local" : undefined}>
      <button
        type="button"
        className="cs-trigger"
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
        <span className={selected ? "cs-trigger-label" : "cs-trigger-label cs-placeholder"}>
          {selected?.label ?? placeholder ?? "—"}
        </span>
        <span className="cs-trigger-caret">▾</span>
      </button>
      {open && (
        <div className="cs-dropdown" role="listbox" tabIndex={-1} onKeyDown={listProps.onKeyDown}>
          {options.map((opt) => {
            const navId = optionNavId(opt.value);
            return (
            <div
              key={opt.value}
              className={`cs-option ${opt.value === value ? "active" : ""}`}
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
      )}
    </div>
  );
}

function optionNavId(value: string) {
  return `option:${value}`;
}
