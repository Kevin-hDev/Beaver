import { useTranslation } from "react-i18next";
import { Spinner } from "@/components/ui/icons";
import type { RetryIndicatorState } from "@/types/agent";
import "./retry-indicator.css";

interface RetryIndicatorProps {
  indicator?: RetryIndicatorState | null;
}

export function RetryIndicator({ indicator }: RetryIndicatorProps) {
  const { t } = useTranslation();
  if (!indicator) return null;

  const label = t(indicator.reasonKey);
  const count = `${indicator.attempt}/${indicator.maxAttempts}`;

  return (
    <span className="ri-root" role="status" aria-live="polite" title={`${label} ${count}`}>
      <span className="ri-label">{label} {count}</span>
      <Spinner className="ri-spinner" size="var(--icon-md)" weight="bold" />
    </span>
  );
}
