import { CaretRight, Check } from "@/components/ui/icons";
import type { useLocalListNavigation } from "@/hooks/use-local-list-navigation";
import type { SelectGroup, SelectOption } from "./settings-select";

type NavApi = ReturnType<typeof useLocalListNavigation>;

interface SettingsSelectListProps {
  filtered: SelectOption[] | null;
  groups?: SelectGroup[];
  options?: SelectOption[];
  collapsed: Record<string, boolean>;
  value: string;
  activate: NavApi["activate"];
  getItemRef: NavApi["getItemRef"];
  isActive: NavApi["isActive"];
  onItemKeyDown: NavApi["listProps"]["onKeyDown"];
  onSelect: (value: string) => void;
  onToggleGroup: (label: string) => void;
}

export function SettingsSelectList({
  filtered,
  groups,
  options,
  collapsed,
  value,
  activate,
  getItemRef,
  isActive,
  onItemKeyDown,
  onSelect,
  onToggleGroup,
}: SettingsSelectListProps) {
  const renderOption = (opt: SelectOption) => {
    const navId = optionNavId(opt.value);
    return (
      <div
        key={opt.value}
        className={`ss-option ${opt.value === value ? "active" : ""} ${opt.dimmed ? "ss-option-dimmed" : ""}`}
        role="button"
        ref={getItemRef(navId)}
        tabIndex={isActive(navId) ? 0 : -1}
        data-local-nav-item="true"
        data-local-nav-active={isActive(navId) ? "true" : undefined}
        onFocus={() => activate(navId)}
        onMouseEnter={() => activate(navId)}
        onClick={() => onSelect(opt.value)}
        onKeyDown={onItemKeyDown}
      >
        <div className="ss-option-check">
          {opt.value === value && <Check size="var(--icon-sm)" weight="bold" />}
        </div>
        {opt.icon}
        <span className="ss-option-label">{opt.label}</span>
      </div>
    );
  };

  if (filtered) {
    if (filtered.length === 0) return <div className="ss-empty">--</div>;
    return <>{filtered.map(renderOption)}</>;
  }

  if (groups) {
    return (
      <>
        {groups.map((group) => {
          const isCollapsed = collapsed[group.label] ?? true;
          const navId = groupNavId(group.label);
          return (
            <div key={group.label} className="ss-group">
              <div
                className="ss-group-header"
                role="button"
                ref={getItemRef(navId)}
                tabIndex={isActive(navId) ? 0 : -1}
                data-local-nav-item="true"
                data-local-nav-active={isActive(navId) ? "true" : undefined}
                onFocus={() => activate(navId)}
                onMouseEnter={() => activate(navId)}
                onClick={() => onToggleGroup(group.label)}
                onKeyDown={onItemKeyDown}
              >
                <CaretRight
                  size="var(--icon-xs)"
                  weight="bold"
                  className={`ss-group-caret ${isCollapsed ? "" : "open"}`}
                />
                <span>{group.label}</span>
                <span className="ss-group-count">{group.options.length}</span>
              </div>
              {!isCollapsed && sortedOptions(group.options).map(renderOption)}
            </div>
          );
        })}
      </>
    );
  }

  if (options) {
    if (options.length === 0) return <div className="ss-empty">--</div>;
    return <>{options.map(renderOption)}</>;
  }

  return <div className="ss-empty">--</div>;
}

export function optionNavId(value: string) {
  return `option:${value}`;
}

export function groupNavId(label: string) {
  return `group:${label}`;
}

export const sortedOptions = (opts: SelectOption[]) =>
  [...opts].sort((a, b) => (a.dimmed ? 1 : 0) - (b.dimmed ? 1 : 0));
