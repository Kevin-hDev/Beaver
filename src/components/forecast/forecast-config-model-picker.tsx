import { useMemo, useState } from "react";
import { ChevronDown } from "@/components/ui/icons";
import { useTranslation } from "react-i18next";
import { useLocalListNavigation, type LocalListNavItem } from "@/hooks/use-local-list-navigation";
import {
  getForecastHardwareKey,
  getForecastModelSummaryKey,
  groupForecastModels,
  type ForecastModelEntry,
} from "./forecast-model-meta";
import "./forecast-config-model-picker.css";

interface ForecastConfigModelPickerProps {
  models: ForecastModelEntry[];
  selectedId: string;
  onSelect: (id: string) => void;
}

export function ForecastConfigModelPicker({
  models,
  selectedId,
  onSelect,
}: ForecastConfigModelPickerProps) {
  const { t } = useTranslation();
  const groups = useMemo(() => groupForecastModels(models), [models]);
  const [openFamilies, setOpenFamilies] = useState<string[]>(["chronos", "timegpt"]);

  const toggleFamily = (familyId: string) => {
    setOpenFamilies((current) =>
      current.includes(familyId)
        ? current.filter((id) => id !== familyId)
        : [...current, familyId]
    );
  };
  const navItems = useMemo<LocalListNavItem[]>(() => groups.flatMap((group) => {
    const isOpen = openFamilies.includes(group.id);
    const groupItem: LocalListNavItem = {
      id: groupNavId(group.id),
      onSelect: () => toggleFamily(group.id),
      onArrowRight: isOpen ? undefined : () => toggleFamily(group.id),
      onArrowLeft: isOpen ? () => toggleFamily(group.id) : undefined,
    };
    const modelItems = isOpen ? group.models.map((model) => ({
      id: modelNavId(model.id),
      onSelect: () => onSelect(model.id),
    })) : [];
    return [groupItem, ...modelItems];
  }), [groups, onSelect, openFamilies]);
  const { activate, getItemRef, isActive, listProps } = useLocalListNavigation({
    items: navItems,
    selectedId: modelNavId(selectedId),
  });

  return (
    <div className="fcmp-root" data-keyboard-scope="local">
      {groups.length === 0 && (
        <div className="fcmp-empty">{t("forecast.models.noneAvailable")}</div>
      )}
      {groups.map((group) => {
        const isOpen = openFamilies.includes(group.id);
        const groupNav = groupNavId(group.id);
        return (
          <div key={group.id} className="fcmp-group">
            <button
              className="fcmp-group-btn"
              type="button"
              ref={getItemRef(groupNav)}
              tabIndex={isActive(groupNav) ? 0 : -1}
              data-local-nav-item="true"
              data-local-nav-active={isActive(groupNav) ? "true" : undefined}
              onFocus={() => activate(groupNav)}
              onMouseEnter={() => activate(groupNav)}
              onKeyDown={listProps.onKeyDown}
              onClick={() => toggleFamily(group.id)}
            >
              <span className="fcmp-group-label">{t(group.titleKey)}</span>
              <span className="fcmp-group-count">{group.models.length}</span>
              <ChevronDown
                size="var(--icon-sm)"
                className={`fcmp-group-chevron ${isOpen ? "is-open" : ""}`}
              />
            </button>
            <div className={`fcmp-group-body ${isOpen ? "is-open" : ""}`}>
              {group.models.map((model) => {
                const active = selectedId === model.id;
                const modelNav = modelNavId(model.id);
                const summaryKey = getForecastModelSummaryKey(model.id);
                const summary = t(summaryKey);
                return (
                  <button
                    key={model.id}
                    className={`fcmp-model ${active ? "is-active" : ""}`}
                    type="button"
                    ref={getItemRef(modelNav)}
                    tabIndex={isActive(modelNav) ? 0 : -1}
                    data-local-nav-item="true"
                    data-local-nav-active={isActive(modelNav) ? "true" : undefined}
                    onFocus={() => activate(modelNav)}
                    onMouseEnter={() => activate(modelNav)}
                    onKeyDown={listProps.onKeyDown}
                    onClick={() => onSelect(model.id)}
                  >
                    <span className="fcmp-radio" aria-hidden="true" />
                    <span className="fcmp-model-main">
                      <span className="fcmp-model-name">{model.display_name}</span>
                      <span className="fcmp-model-summary">
                        {summary === summaryKey ? model.display_name : summary}
                      </span>
                    </span>
                    <span className="fcmp-model-side">
                      <span className="fcmp-model-hardware">
                        {t(getForecastHardwareKey(model))}
                      </span>
                      <span className="fcmp-model-flags">
                        {model.capabilities?.past_covariates ? t("forecast.models.capabilities.context") : ""}
                        {model.capabilities?.past_covariates && model.capabilities?.future_covariates ? " · " : ""}
                        {model.capabilities?.future_covariates ? t("forecast.models.capabilities.futureContext") : ""}
                      </span>
                    </span>
                  </button>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}

function groupNavId(id: string) {
  return `group:${id}`;
}

function modelNavId(id: string) {
  return `model:${id}`;
}
