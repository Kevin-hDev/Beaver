import { useTranslation } from "react-i18next";
import "./error-bubble.css";

interface ErrorBubbleProps {
  message: string;
  isConnection?: boolean;
  diagnosticSummary?: string;
  onRetry?: () => void;
}

export function ErrorBubble({
  message,
  isConnection,
  diagnosticSummary,
  onRetry,
}: ErrorBubbleProps) {
  const { t } = useTranslation();
  const visibleMessage = message === "ollama_connection_lost"
    ? t("errors.ollamaConnectionLost")
    : message;
  const canRetry = !!onRetry && !isConnection;

  return (
    <div className="eb-root" role="alert">
      <div className="eb-copy">
        <div className="eb-message">{visibleMessage}</div>
        {diagnosticSummary && <div className="eb-diagnostic">{diagnosticSummary}</div>}
      </div>
      {canRetry && (
        <button type="button" className="eb-retry" onClick={onRetry}>
          {t("agentLocal.retry.button")}
        </button>
      )}
    </div>
  );
}
