import { useState, useRef, useCallback, useMemo } from "react";
import { createPortal } from "react-dom";
import { CaretDown, MagnifyingGlass } from "@/components/ui/icons";
import { useClickOutside } from "@/hooks/use-click-outside";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
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
  /* Cale la largeur du bouton sur le libellé le plus long de la liste, au lieu
     de la largeur commune. Réservé aux listes dont les libellés identifient la
     valeur choisie — un nom de modèle tronqué ne dit plus lequel est actif. */
  fitLongestOption?: boolean;
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
  fitLongestOption = false,
}: SettingsSelectProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const ref = useRef<HTMLDivElement>(null);

  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  const close = useCallback(() => {
    setOpen(false);
    setQuery("");
  }, []);

  /* Le panneau est porté hors de la page de Réglages : le fondu posé sous le
     titre figé est un masque, et un masque sur un ancêtre interdit tout flou à
     l'intérieur — la liste devenait transparente et laissait lire le texte
     dessous. La carte le coupait aussi à son bord. Aligné à gauche du bouton :
     aligné à droite, il partait vers la gauche et recouvrait ce qui le précède,
     le champ de recherche des chats archivés par exemple. */
  const { anchorRef, floatingRef, floatingStyle } =
    useFloatingMenuPosition(open, "left", 4, placement, true);
  useClickOutside(ref, close, floatingRef);
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

  const panel = open && !disabled ? (
    <div
      ref={floatingRef}
      style={floatingStyle}
      className={`ss-panel ${groups ? "ss-panel-fixed" : ""}`}
      data-keyboard-scope="local"
    >
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
      <div className="ss-panel-scroll">
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
    </div>
  ) : null;

  return (
    <div
      className={`ss-wrap ss-${placement} ${fitLongestOption ? "ss-fit" : ""} ${open ? "open" : ""} ${disabled ? "disabled" : ""}`}
      data-keyboard-scope={open ? "local" : undefined}
      ref={ref}
    >
      <div
        className="btn btn-sm btn-secondary btn-select ss-trigger"
        role="button"
        tabIndex={disabled ? -1 : 0}
        ref={(node) => { anchorRef.current = node; }}
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
        <span className="ss-trigger-text">
          <span className={`ss-trigger-label ${isOverflowing ? "is-overflowing" : ""}`}>
            {displayLabel}
          </span>
          {fitLongestOption && (
            /* Mesureur : tous les libellés empilés dans la même cellule que le
               libellé courant, invisibles et de hauteur nulle. C'est le plus
               long qui impose sa largeur, donc elle ne change pas d'une
               sélection à l'autre. */
            <span className="ss-trigger-sizer" aria-hidden="true">
              {allOptions.map((option) => (
                <span key={option.value}>{option.label}</span>
              ))}
            </span>
          )}
        </span>
        <CaretDown size="var(--icon-sm)" weight="bold" className="ss-trigger-icon" />
      </div>

      {panel ? createPortal(panel, floatingMenuPortalRoot()) : null}
    </div>
  );
}
