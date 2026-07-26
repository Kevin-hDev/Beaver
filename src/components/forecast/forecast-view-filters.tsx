import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { ChevronDown } from "@/components/ui/icons";
import { useTranslation } from "react-i18next";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
import { focusLocalListItem, useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";
import type {
  ForecastLayerGroup,
  ForecastLayerState,
} from "./forecast-layer-matrix";
import { FilterGroup, FilterItem } from "./forecast-view-filter-items";
import { buildForecastFilterChips } from "./forecast-filter-chip";
import "./forecast-view-filters.css";

interface ForecastViewFiltersProps {
  groups: ForecastLayerGroup[];
  layers: ForecastLayerState;
  onChange: (layers: ForecastLayerState) => void;
}

export function ForecastViewFilters({
  groups,
  layers,
  onChange,
}: ForecastViewFiltersProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [openGroups, setOpenGroups] = useState<string[]>([]);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const pendingFocusDirection = useRef<1 | -1>(1);
  const { anchorRef, floatingRef, floatingStyle } =
    useFloatingMenuPosition(open, "right", 6, "auto");

  useEffect(() => {
    if (!open) return;
    const handlePointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (
        !rootRef.current?.contains(target)
        && !floatingRef.current?.contains(target)
      ) {
        setOpen(false);
        setOpenGroups([]);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
        setOpenGroups([]);
      }
    };
    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [floatingRef, open]);

  const toggleGroup = useCallback((groupId: string) => {
    setOpenGroups((current) =>
      current.includes(groupId)
        ? current.filter((id) => id !== groupId)
        : [...current, groupId]
    );
  }, []);

  const navItems = useMemo<LocalListNavItem[]>(() => groups.flatMap((group) => {
    const expanded = openGroups.includes(group.id);
    const groupNavId = `group:${group.id}`;
    const children = expanded ? group.items.map((item) => ({
      id: `item:${item.id}`,
      disabled: !item.interactive,
      onSelect: () => onChange({ ...layers, [item.id]: !layers[item.id] }),
    })) : [];
    return [
      {
        id: groupNavId,
        onSelect: () => toggleGroup(group.id),
        onArrowRight: () => setOpenGroups((current) => current.includes(group.id) ? current : [...current, group.id]),
        onArrowLeft: () => setOpenGroups((current) => current.filter((id) => id !== group.id)),
      },
      ...children,
    ];
  }), [groups, layers, onChange, openGroups, toggleGroup]);

  const nav = useLocalListNavigation({
    items: navItems,
    enabled: open,
    onEscape: () => {
      setOpen(false);
      setOpenGroups([]);
    },
  });

  const chips = useMemo(() => buildForecastFilterChips(groups), [groups]);

  useEffect(() => {
    if (open) focusLocalListItem(floatingRef.current, pendingFocusDirection.current);
  }, [floatingRef, open]);

  const panel = open ? (
    <div
      ref={floatingRef}
      className="fcf-panel"
      role="menu"
      tabIndex={-1}
      style={floatingStyle}
      data-keyboard-scope="local"
      onKeyDown={nav.listProps.onKeyDown}
    >
      {groups.map((group) => (
        <FilterGroup
          key={group.id}
          groupId={group.id}
          open={openGroups.includes(group.id)}
          title={t(group.titleKey)}
          onToggle={toggleGroup}
          nav={nav}
        >
          {group.items.length > 0 ? (
            group.items.map((item) => (
              <FilterItem
                key={item.id}
                navId={`item:${item.id}`}
                label={item.label}
                checked={Boolean(layers[item.id])}
                disabled={!item.interactive}
                chip={item.interactive ? chips.get(item.id) : undefined}
                nav={nav}
                onToggle={() =>
                  item.interactive
                    ? onChange({ ...layers, [item.id]: !layers[item.id] })
                    : undefined
                }
              />
            ))
          ) : (
            <div className="fcf-empty">{group.emptyKey ? t(group.emptyKey) : ""}</div>
          )}
        </FilterGroup>
      ))}
    </div>
  ) : null;

  return (
    <div ref={rootRef} className="fcf-root" data-keyboard-scope="local">
      <button
        ref={(node) => { anchorRef.current = node; }}
        className={`btn btn-sm btn-secondary fcf-trigger ${open ? "is-open" : ""}`}
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => {
          setOpen((current) => {
            const next = !current;
            if (!next) setOpenGroups([]);
            return next;
          });
        }}
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          event.stopPropagation();
          pendingFocusDirection.current = event.key === "ArrowDown" ? 1 : -1;
          if (open) focusLocalListItem(floatingRef.current, pendingFocusDirection.current);
          else setOpen(true);
        }}
      >
        <span>{t("forecast.view.filters.button")}</span>
        <ChevronDown size="var(--icon-sm)" className={`fcf-chevron ${open ? "is-open" : ""}`} />
      </button>
      {panel ? createPortal(panel, floatingMenuPortalRoot()) : null}
    </div>
  );
}
