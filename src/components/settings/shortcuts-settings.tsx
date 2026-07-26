import { useTranslation } from "react-i18next";
import { SettingsCard } from "./settings-card";
import { MOD_LABEL, ALT_LABEL } from "@/lib/platform";
import "./shortcuts-settings.css";

interface Shortcut {
  i18n: string;
  keys: string[];
}

const SHORTCUTS: Shortcut[] = [
  { i18n: "settings.shortcuts.toggleTerminal", keys: [MOD_LABEL, "J"] },
  { i18n: "settings.shortcuts.toggleSidebar", keys: [MOD_LABEL, "B"] },
  { i18n: "settings.shortcuts.goBack", keys: [MOD_LABEL, "◀"] },
  { i18n: "settings.shortcuts.goForward", keys: [MOD_LABEL, "▶"] },
  { i18n: "settings.shortcuts.newSession", keys: [ALT_LABEL, MOD_LABEL, "N"] },
  { i18n: "settings.shortcuts.searchDialog", keys: [MOD_LABEL, "G"] },
  { i18n: "settings.shortcuts.togglePreview", keys: [ALT_LABEL, MOD_LABEL, "J"] },
];

export function ShortcutsSettings() {
  const { t } = useTranslation();

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 600, width: "100%", margin: "0 auto" }}>
        <h2 style={{
          fontSize: "var(--text-xl)",
          fontWeight: 700,
          color: "var(--ink)",
          marginBottom: 28,
        }}>
          {t("settings.tabs.shortcuts")}
        </h2>

        <SettingsCard>
          {SHORTCUTS.map((shortcut) => (
            <div key={shortcut.i18n} className="shortcut-row">
              <span className="shortcut-label">{t(shortcut.i18n)}</span>
              <span className="shortcut-keys">
                {shortcut.keys.map((key, i) => (
                  <span key={i}>
                    <kbd className="shortcut-key">{key}</kbd>
                    {i < shortcut.keys.length - 1 && (
                      <span className="shortcut-plus">+</span>
                    )}
                  </span>
                ))}
              </span>
            </div>
          ))}
        </SettingsCard>
      </div>
    </div>
  );
}
