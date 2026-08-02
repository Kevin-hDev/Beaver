import { useTranslation } from "react-i18next";
import "./vault-error-banner.css";

interface VaultErrorBannerProps {
  onDismiss: () => void;
}

export function VaultErrorBanner({ onDismiss }: VaultErrorBannerProps) {
  const { t } = useTranslation();
  return (
    <button type="button" className="veb-root" onClick={onDismiss}>
      {t("errors.keyringFailed")}
    </button>
  );
}
