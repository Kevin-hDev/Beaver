import { useState, useRef, useCallback, useMemo } from "react";
import { CaretDown, MagnifyingGlass } from "@/components/ui/icons";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";
import { SettingsSelectList, groupNavId, optionNavId, sortedOptions } from "./settings-select-list";
import "./settings-select.css";

export interface SelectOption {
  value: string;
  label: string;
  icon?: React.ReactNode;
  dimmed?: boolean;
}

export interface SelectGroup {
  label: string;
  options: SelectOption[];
}

interface SettingsSelectProps {
  options?: SelectOption[];
  groups?: SelectGroup[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  searchable?: boolean;
  searchPlaceholder?: string;
  disabled?: boolean;
  placement?: "above" | "below";
}

const EMPTY_OPTIONS: SelectOption[] = [];

export function SettingsSelect({
  options,
  groups,
  value,
  onChange,
  placeholder,
  searchable,
  searchPlaceholder,
  disabled,
  placement = "below",
}: SettingsSelectProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const ref = useRef<HTMLDivElement>(null);

  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  const close = useCallback(() => {
    setOpen(false);
    setQuery("");
  }, []);

  useClickOutside(ref, close);
  useKeyboard({ onEscape: open ? close : undefined });

  const allOptions = useMemo(() => {
    if (options) return options;
    if (!groups) return [];
    return groups.flatMap((g) => g.options);
  }, [options, groups]);

  const filtered = useMemo(() => {
    if (!searchable || !query) return null;
    const q = query.toLowerCase();
    return allOptions.filter((o) => o.label.toLowerCase().includes(q));
  }, [allOptions, query, searchable]);

  const current = allOptions.find((o) => o.value === value);
  const fallbackLabel = value && value.includes(":") ? value.split(":").slice(1).join(":") : value;
  const displayLabel = current?.label ?? (value ? fallbackLabel : placeholder) ?? "—";
  const isOverflowing = displayLabel.length > 20;

  const handleSelect = useCallback((val: string) => {
    if (disabled) return;
    onChange(val);
    close();
  }, [close, disabled, onChange]);

  const toggleGroup = useCallback((label: string) => {
    setCollapsed((prev) => ({ ...prev, [label]: !(prev[label] ?? true) }));
  }, []);

  const visibleOptions = filtered ?? options ?? EMPTY_OPTIONS;
  const navItems = useMemo<LocalListNavItem[]>(() => {
    if (filtered || options) {
      return visibleOptions.map((opt) => ({
        id: optionNavId(opt.value),
        onSelect: () => handleSelect(opt.value),
      }));
    }
    return (groups ?? []).flatMap((group) => {
      const isCollapsed = collapsed[group.label] ?? true;
      const groupItem: LocalListNavItem = {
        id: groupNavId(group.label),
        onSelect: () => toggleGroup(group.label),
        onArrowRight: isCollapsed ? () => toggleGroup(group.label) : undefined,
        onArrowLeft: isCollapsed ? undefined : () => toggleGroup(group.label),
      };
      const optionItems = isCollapsed ? [] : sortedOptions(group.options).map((opt) => ({
        id: optionNavId(opt.value),
        onSelect: () => handleSelect(opt.value),
      }));
      return [groupItem, ...optionItems];
    });
  }, [collapsed, filtered, groups, handleSelect, options, toggleGroup, visibleOptions]);

  const selectedNavId = navItems.some((item) => item.id === optionNavId(value)) ? optionNavId(value) : null;
  const { activate, getItemRef, isActive, listProps } = useLocalListNavigation({
    items: navItems,
    enabled: open && !disabled,
    selectedId: selectedNavId,
    onEscape: close,
  });

  return (
    <div
      className={`ss-wrap ss-${placement} ${open ? "open" : ""} ${disabled ? "disabled" : ""}`}
      data-keyboard-scope={open ? "local" : undefined}
      ref={ref}
    >
      <div
        className="ss-trigger"
        role="button"
        tabIndex={disabled ? -1 : 0}
        onClick={() => !disabled && setOpen(!open)}
        onKeyDown={(event) => {
          if (disabled) return;
          const directionKey = placement === "above" ? "ArrowUp" : "ArrowDown";
          if (!open && (
            event.key === "Enter"
            || event.key === " "
            || event.key === directionKey
          )) {
            setOpen(true);
            return;
          }
          if (open) listProps.onKeyDown(event);
        }}
        title={isOverflowing ? displayLabel : undefined}
      >
        <span className={`ss-trigger-label ${isOverflowing ? "is-overflowing" : ""}`}>
          {displayLabel}
        </span>
        <CaretDown size="var(--icon-sm)" weight="bold" className="ss-trigger-icon" />
      </div>

      {open && !disabled && (
        <div className={`ss-panel ${groups ? "ss-panel-fixed" : ""}`}>
          {searchable && (
            <div className="ss-search">
              <MagnifyingGlass size="var(--icon-sm)" />
              <input
                autoFocus
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={listProps.onKeyDown}
                placeholder={searchPlaceholder ?? ""}
              />
            </div>
          )}
          <SettingsSelectList
            filtered={filtered}
            groups={groups}
            options={options}
            collapsed={collapsed}
            value={value}
            activate={activate}
            getItemRef={getItemRef}
            isActive={isActive}
            onItemKeyDown={listProps.onKeyDown}
            onSelect={handleSelect}
            onToggleGroup={toggleGroup}
          />
        </div>
      )}
    </div>
  );
}
