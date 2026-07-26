import { useTranslation } from "react-i18next";
import { CaretRight } from "@/components/ui/icons";

interface OnboardingWelcomeProps {
  onNext: () => void;
}

export function OnboardingWelcome({ onNext }: OnboardingWelcomeProps) {
  const { t } = useTranslation();

  return (
    <div className="ob-page ob-page-centered">
      <span className="ob-brand-castor" aria-hidden="true" />
      <div className="ob-copy">
        <h1 className="ob-title">{t("onboarding.welcome.title")}</h1>
        <p className="ob-description">{t("onboarding.welcome.description")}</p>
      </div>
      <button type="button" className="ob-primary-btn" onClick={onNext}>
        {t("onboarding.welcome.getStarted")}
        <CaretRight size="var(--icon-sm)" weight="bold" />
      </button>
    </div>
  );
}
