import { useTranslation } from "react-i18next";
import { InlineActivityIndicator } from "./inline-activity-indicator";
import "./clone-summary-run-button.css";

interface CloneSummaryRunButtonProps {
  onClick: () => void;
}

export function CloneSummaryRunButton({ onClick }: CloneSummaryRunButtonProps) {
  return (
    <button type="button" className="csr-summary-run" onClick={onClick}>
      <CloneSummaryRunIndicator />
    </button>
  );
}

export function CloneSummaryRunIndicator({ className = "" }: { className?: string }) {
  const { t } = useTranslation();
  return (
    <InlineActivityIndicator className={className}>
      {t("agentLocal.clone.running")}
    </InlineActivityIndicator>
  );
}
