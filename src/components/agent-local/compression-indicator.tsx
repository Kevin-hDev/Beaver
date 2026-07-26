import { useTranslation } from "react-i18next";
import "./compression-indicator.css";

interface CompressionIndicatorProps {
  label?: string;
}

export function CompressionIndicator({ label }: CompressionIndicatorProps) {
  const { t } = useTranslation();

  return (
    <div className="compression-indicator" role="status" aria-live="polite">
      <span className="compression-line" aria-hidden="true" />
      <span className="compression-label">{label ?? t("agentLocal.compression")}</span>
      <span className="compression-line" aria-hidden="true" />
    </div>
  );
}
