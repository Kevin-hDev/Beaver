import type { KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { Check, Star } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import type { AvailableModel } from "@/hooks/use-available-models";
import type { useLocalListNavigation } from "@/hooks/use-local-list-navigation";
import "./model-selector-disabled.css";

type NavApi = ReturnType<typeof useLocalListNavigation>;

interface ModelSelectorItemProps {
  model: AvailableModel;
  isSelected: boolean;
  isFav: boolean;
  navId: string;
  activate: NavApi["activate"];
  getItemRef: NavApi["getItemRef"];
  isNavActive: NavApi["isActive"];
  onItemKeyDown: (event: KeyboardEvent) => void;
  onSelect: (model: string, provider: string) => void;
  onToggleFav: (provider: string, model: string) => void;
}

export function ModelSelectorItem({
  model: m,
  isSelected,
  isFav,
  navId,
  activate,
  getItemRef,
  isNavActive,
  onItemKeyDown,
  onSelect,
  onToggleFav,
}: ModelSelectorItemProps) {
  const { t } = useTranslation();
  const isPaid = !m.is_free && !m.is_local;
  const disabled = m.disabled === true;
  return (
    <div
      className={`ms-item ${isSelected ? "active" : ""} ${isPaid ? "ms-item-paid" : ""} ${disabled ? "ms-item-disabled" : ""}`}
      role="button"
      aria-disabled={disabled || undefined}
      ref={getItemRef(navId)}
      tabIndex={!disabled && isNavActive(navId) ? 0 : -1}
      data-local-nav-item={disabled ? undefined : "true"}
      data-local-nav-active={!disabled && isNavActive(navId) ? "true" : undefined}
      onFocus={() => { if (!disabled) activate(navId); }}
      onMouseEnter={() => { if (!disabled) activate(navId); }}
      onKeyDown={disabled ? undefined : onItemKeyDown}
      onClick={() => { if (!disabled) onSelect(m.id, m.provider_id); }}
    >
      <span
        className={`ms-star ${isFav ? "ms-star-on" : ""}`}
        role={disabled ? undefined : "button"}
        tabIndex={disabled ? -1 : 0}
        aria-hidden={disabled || undefined}
        onClick={(event) => {
          event.stopPropagation();
          if (!disabled) onToggleFav(m.provider_id, m.id);
        }}
        onKeyDown={(event) => {
          if (!disabled && (event.key === "Enter" || event.key === " ")) {
            event.stopPropagation();
            onToggleFav(m.provider_id, m.id);
          }
        }}
      >
        <Star size="var(--icon-xs)" weight={isFav ? "fill" : "regular"} />
      </span>
      <span className="ms-item-name">{m.display_name ?? m.id}</span>
      <span className="ms-item-right">
        {m.supports_vision && (
          <Tooltip label={t("settings.llm.vision")}>
            <span className="ms-badge-vision">V</span>
          </Tooltip>
        )}
        {m.supports_tools && (
          <Tooltip label={t("settings.llm.tools")}>
            <span className="ms-badge-tools">T</span>
          </Tooltip>
        )}
        {(m.disabled_hint ?? m.hint) && (
          <span className="ms-hint">{m.disabled_hint ?? m.hint}</span>
        )}
        {isSelected && <Check size="var(--icon-xs)" />}
      </span>
    </div>
  );
}
