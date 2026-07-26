import type { ReactNode } from "react";
import { ChevronDown } from "@/components/ui/icons";
import type { useLocalListNavigation } from "@/hooks/use-local-list-navigation";
import type { ForecastFilterChip } from "./forecast-filter-chip";

type NavApi = ReturnType<typeof useLocalListNavigation>;

interface FilterGroupProps {
  groupId: string;
  open: boolean;
  title: string;
  nav: NavApi;
  onToggle: (groupId: string) => void;
  children: ReactNode;
}

export function FilterGroup({
  groupId,
  open,
  title,
  nav,
  onToggle,
  children,
}: FilterGroupProps) {
  const navId = `group:${groupId}`;
  return (
    <div className="fcf-group">
      <button
        ref={nav.getItemRef(navId)}
        className="fcf-group-btn"
        type="button"
        data-local-nav-item="true"
        data-local-nav-active={nav.isActive(navId) ? "true" : undefined}
        tabIndex={nav.isActive(navId) ? 0 : -1}
        onFocus={() => nav.activate(navId)}
        onMouseEnter={() => nav.activate(navId)}
        onKeyDown={nav.listProps.onKeyDown}
        onClick={() => onToggle(groupId)}
      >
        <span className="fcf-group-title">{title}</span>
        <ChevronDown size="var(--icon-sm)" className={`fcf-group-chevron ${open ? "is-open" : ""}`} />
      </button>
      <div className={`fcf-group-items ${open ? "is-open" : ""}`}>
        <div className="fcf-group-content">{children}</div>
      </div>
    </div>
  );
}

interface FilterItemProps {
  navId: string;
  label: string;
  checked: boolean;
  disabled?: boolean;
  chip?: ForecastFilterChip;
  nav: NavApi;
  onToggle?: () => void;
}

export function FilterItem({
  navId,
  label,
  checked,
  disabled,
  chip,
  nav,
  onToggle,
}: FilterItemProps) {
  return (
    <button
      ref={nav.getItemRef(navId)}
      className={`fcf-item ${disabled ? "is-disabled" : ""}`}
      type="button"
      role="checkbox"
      aria-checked={checked}
      aria-disabled={disabled ? "true" : undefined}
      disabled={disabled}
      data-local-nav-item="true"
      data-local-nav-active={nav.isActive(navId) ? "true" : undefined}
      tabIndex={nav.isActive(navId) ? 0 : -1}
      onFocus={() => nav.activate(navId)}
      onMouseEnter={() => nav.activate(navId)}
      onKeyDown={nav.listProps.onKeyDown}
      onClick={() => onToggle?.()}
    >
      <span className="fcf-checkbox" aria-hidden="true">
        {checked ? <span className="fcf-checkbox-mark" /> : null}
      </span>
      {chip ? (
        <span
          className={`fcf-chip ${chip.shape === "band" ? "fcf-chip-band" : ""}`}
          style={{ background: chip.color }}
          aria-hidden="true"
        />
      ) : null}
      <span className="fcf-item-label">{label}</span>
    </button>
  );
}
