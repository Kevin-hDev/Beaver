import { useTranslation } from "react-i18next";
import { useResolvedSettingsSections } from "./settings-sections";
import type { SubTabDef } from "./settings-sections";
import type { SettingsSubTab } from "@/types/navigation";
import { localizedText } from "@/features/extension-ui/standard/localized-text";

interface SettingsSubTabListProps {
  active: SettingsSubTab;
  onSelect: (id: SettingsSubTab) => void;
}

export function SettingsSubTabList({ active, onSelect }: SettingsSubTabListProps) {
  const { t } = useTranslation();
  const sections = useResolvedSettingsSections();
  return (
    <div className="settings-subtab-list">
      {sections.map((section) => (
        <div key={section.i18n} className="settings-subtab-group">
          {/* L'en-tête décrit le groupe sans être une destination : ni rôle de
              bouton, ni tabIndex, sous peine que la navigation aux flèches et
              la tabulation s'arrêtent sur une ligne qui n'ouvre rien. */}
          <div className="settings-subtab-heading">{t(section.i18n)}</div>
          {section.tabs.map((tab) => (
            <SubTabItem
              key={tab.id}
              tab={tab}
              active={active === tab.id}
              onSelect={onSelect}
            />
          ))}
        </div>
      ))}
    </div>
  );
}

interface SubTabItemProps {
  tab: SubTabDef;
  active: boolean;
  onSelect: (id: SettingsSubTab) => void;
}

function SubTabItem({ tab, active, onSelect }: SubTabItemProps) {
  const { t } = useTranslation();
  const label = tab.i18n ? t(tab.i18n) : localizedText(tab.label!);
  return (
    <div
      role="button"
      tabIndex={active ? 0 : -1}
      data-nav-active={active ? "true" : undefined}
      onClick={() => onSelect(tab.id)}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect(tab.id);
        }
      }}
      className={`settings-subtab${active ? " active" : ""}`}
    >
      <tab.icon className="settings-subtab-icon" />
      <span className="settings-subtab-label">{label}</span>
    </div>
  );
}
