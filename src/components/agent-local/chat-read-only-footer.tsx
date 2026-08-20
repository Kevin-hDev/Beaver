import { ErrorBubble } from "./error-bubble";
import { ScrollBottomButton } from "./scroll-bottom-button";

interface ChatReadOnlyFooterProps {
  diagnosticSummary?: string;
  error?: string;
  isConnectionError?: boolean;
  onScrollBottom: () => void;
  showError: boolean;
  showScrollButton: boolean;
}

export function ChatReadOnlyFooter({
  diagnosticSummary,
  error,
  isConnectionError,
  onScrollBottom,
  showError,
  showScrollButton,
}: ChatReadOnlyFooterProps) {
  if ((!showError || !error) && !showScrollButton) return null;

  return (
    <div className="chat-input-area">
      <div className="chat-input-column">
        {showError && error && (
          <ErrorBubble
            message={error}
            isConnection={isConnectionError}
            diagnosticSummary={diagnosticSummary}
          />
        )}
        {showScrollButton && (
          <div className="chat-input-anchor">
            <ScrollBottomButton onClick={onScrollBottom} />
          </div>
        )}
      </div>
    </div>
  );
}
