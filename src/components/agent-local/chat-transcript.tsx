import type { RefObject } from "react";
import {
  ChatMessagePanel,
  type ChatMessagePanelProps,
} from "./chat-message-panel";
import { ChatReadOnlyFooter } from "./chat-read-only-footer";

interface ChatTranscriptProps extends ChatMessagePanelProps {
  containerRef: RefObject<HTMLDivElement | null>;
  isAtBottom: boolean;
  onScrollBottom: () => void;
}

export function ChatTranscript({
  containerRef,
  isAtBottom,
  onScrollBottom,
  ...messagePanelProps
}: ChatTranscriptProps) {
  const { chat, readOnly, runtime } = messagePanelProps;

  return (
    <>
      <div className="chat-messages" ref={containerRef}>
        <ChatMessagePanel {...messagePanelProps} />
      </div>
      {readOnly && (
        <ChatReadOnlyFooter
          diagnosticSummary={chat.diagnosticSummary}
          error={chat.error}
          isConnectionError={chat.isConnectionError}
          onScrollBottom={onScrollBottom}
          showError={runtime.showError}
          showScrollButton={!isAtBottom}
        />
      )}
    </>
  );
}
