import { useTranslation } from "react-i18next";
import { SettingsPanel } from "./shell/settings-panel";
import { SettingsCard } from "./settings-card";
import { MOD_LABEL, ALT_LABEL } from "@/lib/platform";
import { APP_SHORTCUTS } from "@/lib/app-shortcuts";
import "./shortcuts-settings.css";

function displayKey(key: string): string {
  if (key === "mod") return MOD_LABEL;
  if (key === "alt") return ALT_LABEL;
  if (key === "shift") return "Shift";
  return key;
}

export function ShortcutsSettings() {
  const { t } = useTranslation();

  return (
    <SettingsPanel title={t("settings.tabs.shortcuts")}>

      <SettingsCard>
        {APP_SHORTCUTS.map((shortcut) => (
          <div key={shortcut.id} className="scs-row">
            <span className="scs-label">{t(shortcut.i18n)}</span>
            <span className="scs-keys">
              {shortcut.keys.map((key, i) => (
                <span key={`${key}-${i}`}>
                  <kbd className="scs-key">{displayKey(key)}</kbd>
                  {i < shortcut.keys.length - 1 && (
                    <span className="scs-plus">+</span>
                  )}
                </span>
              ))}
            </span>
          </div>
        ))}
      </SettingsCard>
    </SettingsPanel>
  );
}
