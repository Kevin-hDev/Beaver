import { useTranslation } from "react-i18next";
import type { ThemeChoice } from "@/hooks/use-theme";
import { CaretRight } from "@/components/ui/icons";
import { SettingsSelect } from "@/components/settings/settings-select";
import { ThemeSelector } from "@/components/settings/theme-selector";
import { LANGUAGE_OPTIONS } from "@/components/settings/general-settings-options";

interface OnboardingPreferencesProps {
  themeChoice: ThemeChoice;
  onThemeChange: (theme: ThemeChoice) => void;
  onNext: () => void;
}

export function OnboardingPreferences({
  themeChoice,
  onThemeChange,
  onNext,
}: OnboardingPreferencesProps) {
  const { t, i18n } = useTranslation();

  const changeLang = (lang: string) => {
    void i18n.changeLanguage(lang);
    localStorage.setItem("clgo-language", lang);
  };

  return (
    <div className="ob-page">
      <div className="ob-copy">
        <h1 className="ob-title">{t("onboarding.preferences.title")}</h1>
        <p className="ob-description">{t("onboarding.preferences.description")}</p>
      </div>

      <div className="ob-settings-panel">
        <div className="ob-field">
          <span className="ob-field-label">{t("settings.general.themeTitle")}</span>
          <ThemeSelector value={themeChoice} onChange={onThemeChange} />
        </div>

        <div className="ob-field">
          <span className="ob-field-label">{t("settings.general.languageTitle")}</span>
          <SettingsSelect
            options={LANGUAGE_OPTIONS}
            value={i18n.language}
            onChange={changeLang}
            placement="above"
          />
        </div>
      </div>

      <button type="button" className="ob-primary-btn" onClick={onNext}>
        {t("onboarding.common.continue")}
        <CaretRight size="var(--icon-sm)" weight="bold" />
      </button>
    </div>
  );
}
