import type { RefObject } from "react";
import {
  ChatMessagePanel,
  type ChatMessagePanelProps,
} from "./chat-message-panel";
import { ChatReadOnlyFooter } from "./chat-read-only-footer";
import { ConversationSearch } from "./conversation-search";
import { useConversationSearch } from "@/hooks/use-conversation-search";

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
  const search = useConversationSearch(chat.messages);

  return (
    <>
      <ConversationSearch
        open={search.open}
        query={search.query}
        activePosition={search.activePosition}
        totalMatches={search.totalMatches}
        focusRequest={search.focusRequest}
        onQueryChange={search.setQuery}
        onMove={search.move}
        onClose={search.close}
      />
      <div className="chat-messages" ref={containerRef}>
        <ChatMessagePanel
          {...messagePanelProps}
          activeSearchMessageId={search.activeMessageId}
        />
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
