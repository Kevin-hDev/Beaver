import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretRight } from "@/components/ui/icons";
import type { AvailableModel } from "@/hooks/use-available-models";
import type { FavoriteModel } from "@/hooks/use-favorite-models";
import { useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";
import { ModelSelectorItem } from "./model-selector-item";

interface Props {
  groups: Map<string, AvailableModel[]>;
  favorites: FavoriteModel[];
  isFavorite: (provider: string, model: string) => boolean;
  onToggleFavorite: (provider: string, model: string) => void;
  selectedModel: string;
  selectedProvider: string;
  onSelect: (model: string, provider: string) => void;
}

export function ModelSelectorList({
  groups,
  favorites,
  isFavorite,
  onToggleFavorite,
  selectedModel,
  selectedProvider,
  onSelect,
}: Props) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const toggle = useCallback((key: string) =>
    setExpanded((s) => {
      const n = new Set(s);
      if (n.has(key)) { n.delete(key); } else { n.add(key); }
      return n;
    }), []);

  const favModels = useMemo(() => favoriteModels(groups, favorites), [favorites, groups]);

  const sortedGroups = useMemo(() => Array.from(groups.entries()), [groups]);
  const navItems = useMemo<LocalListNavItem[]>(() => {
    const items: LocalListNavItem[] = favModels.filter((m) => !m.disabled).map((m) => ({
      id: navModelId("fav", m.provider_id, m.id),
      onSelect: () => onSelect(m.id, m.provider_id),
    }));
    for (const [providerId, models] of sortedGroups) {
      const isOpen = expanded.has(providerId);
      items.push({
        id: navProviderId(providerId),
        onSelect: () => toggle(providerId),
        onArrowRight: isOpen ? undefined : () => toggle(providerId),
        onArrowLeft: isOpen ? () => toggle(providerId) : undefined,
      });
      if (isOpen) {
        for (const m of sortedModels(models)) {
          if (m.disabled) continue;
          items.push({
            id: navModelId("model", m.provider_id, m.id),
            onSelect: () => onSelect(m.id, m.provider_id),
          });
        }
      }
    }
    return items;
  }, [expanded, favModels, onSelect, sortedGroups, toggle]);

  const selectedNavId = navItems.find((item) => item.id.endsWith(`:${selectedProvider}:${selectedModel}`))?.id ?? null;
  const { activate, getItemRef, isActive, listProps } = useLocalListNavigation({ items: navItems, selectedId: selectedNavId });

  if (groups.size === 0 && favModels.length === 0) {
    return <div className="ms-empty">{t("agentLocal.modelEmpty")}</div>;
  }

  return (
    <div {...listProps}>
      {favModels.length > 0 && (
        <div>
          <div className="ms-section ms-section-fav">★ {t("agentLocal.favorites")}</div>
          {favModels.map((m) => (
            <ModelSelectorItem
              key={`fav:${m.provider_id}:${m.id}`}
              navId={navModelId("fav", m.provider_id, m.id)}
              activate={activate}
              getItemRef={getItemRef}
              isNavActive={isActive}
              onItemKeyDown={listProps.onKeyDown}
              model={m}
              isSelected={m.id === selectedModel && m.provider_id === selectedProvider}
              isFav
              onSelect={onSelect}
              onToggleFav={onToggleFavorite}
            />
          ))}
        </div>
      )}

      {sortedGroups.map(([providerId, models]) => {
        const isOpen = expanded.has(providerId);
        const name = models[0]?.provider_name ?? providerId;
        const freeCount = models.filter((m) => m.is_free || m.is_local).length;
        const providerNavId = navProviderId(providerId);
        return (
          <div key={providerId}>
            <div
              className="ms-provider"
              role="button"
              ref={getItemRef(providerNavId)}
              tabIndex={isActive(providerNavId) ? 0 : -1}
              data-local-nav-item="true"
              data-local-nav-active={isActive(providerNavId) ? "true" : undefined}
              onFocus={() => activate(providerNavId)}
              onMouseEnter={() => activate(providerNavId)}
              onKeyDown={listProps.onKeyDown}
              onClick={() => toggle(providerId)}
            >
              <span className="ms-provider-caret">
                {isOpen ? <CaretDown size="var(--icon-xs)" /> : <CaretRight size="var(--icon-xs)" />}
              </span>
              <span className="ms-provider-name">{name}</span>
              <span className="ms-provider-count">
                {freeCount > 0 && freeCount < models.length
                  ? t("agentLocal.freeCount", { free: freeCount, total: models.length })
                  : `${models.length}`}
              </span>
            </div>
            <div className={`ms-provider-body ${isOpen ? "open" : ""}`}>
              <div className="ms-provider-body-inner">
                {sortedModels(models).map((m) => (
                  <ModelSelectorItem
                    key={`${m.provider_id}:${m.id}`}
                    navId={navModelId("model", m.provider_id, m.id)}
                    activate={activate}
                    getItemRef={getItemRef}
                    isNavActive={isActive}
                    onItemKeyDown={listProps.onKeyDown}
                    model={m}
                    isSelected={m.id === selectedModel && m.provider_id === selectedProvider}
                    isFav={isFavorite(m.provider_id, m.id)}
                    onSelect={onSelect}
                    onToggleFav={onToggleFavorite}
                  />
                ))}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

function navProviderId(providerId: string) {
  return `provider:${providerId}`;
}

function navModelId(kind: "fav" | "model", providerId: string, modelId: string) {
  return `${kind}:${providerId}:${modelId}`;
}

function favoriteModels(groups: Map<string, AvailableModel[]>, favorites: FavoriteModel[]) {
  const models: AvailableModel[] = [];
  for (const fav of favorites) {
    const model = groups.get(fav.provider)?.find((item) => item.id === fav.model);
    if (model && !model.disabled) models.push(model);
  }
  return models;
}

function sortedModels(models: AvailableModel[]): AvailableModel[] {
  return [...models].sort((a, b) => {
    const af = a.is_free || a.is_local ? 0 : 1;
    const bf = b.is_free || b.is_local ? 0 : 1;
    if (af !== bf) return af - bf;
    return a.id.localeCompare(b.id);
  });
}
