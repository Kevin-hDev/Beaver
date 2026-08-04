import { useTranslation } from "react-i18next";
import "./welcome-wordmark.css";

interface WelcomeWordmarkProps {
  leaving: boolean;
}

export function WelcomeWordmark({ leaving }: WelcomeWordmarkProps) {
  const { t } = useTranslation();

  return (
    <div className={`wm-wrap ${leaving ? "wm-leaving" : ""}`}>
      <h1 className="wm-title">
        {t("welcome.title")}
        <span className="wm-dot" aria-hidden="true">.</span>
      </h1>
    </div>
  );
}
