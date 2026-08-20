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
  // Le conteneur reste monté : le bouton absolu peut basculer sans déplacer la transcription.
  return (
    <div className="chat-input-area chat-read-only-footer">
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
