import { useTranslation } from "react-i18next";
import { ArrowLeft } from "@/components/ui/icons";

interface SettingsBackButtonProps {
  onClick: () => void;
}

export function SettingsBackButton({ onClick }: SettingsBackButtonProps) {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="icon-btn"
      aria-label={t("common.back")}
      onClick={onClick}
    >
      <ArrowLeft size="var(--icon-md)" />
    </button>
  );
}
