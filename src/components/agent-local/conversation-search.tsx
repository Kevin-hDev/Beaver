import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretUp, X } from "@/components/ui/icons";
import "./conversation-search.css";

interface ConversationSearchProps {
  open: boolean;
  query: string;
  activePosition: number;
  totalMatches: number;
  focusRequest: number;
  onQueryChange: (query: string) => void;
  onMove: (direction: 1 | -1) => void;
  onClose: () => void;
}

export function ConversationSearch({
  open,
  query,
  activePosition,
  totalMatches,
  focusRequest,
  onQueryChange,
  onMove,
  onClose,
}: ConversationSearchProps) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) inputRef.current?.focus();
  }, [focusRequest, open]);

  if (!open) return null;

  return (
    <div className="cfs-root" role="search">
      <input
        ref={inputRef}
        className="cfs-input"
        type="search"
        value={query}
        maxLength={120}
        aria-label={t("conversationSearch.label")}
        placeholder={t("conversationSearch.placeholder")}
        onChange={(event) => onQueryChange(event.target.value)}
        onKeyDown={(event) => {
          if (event.key !== "Enter") return;
          event.preventDefault();
          onMove(event.shiftKey ? -1 : 1);
        }}
      />
      <span className="cfs-count" aria-live="polite">
        {activePosition} / {totalMatches}
      </span>
      <button type="button" className="cfs-action" aria-label={t("conversationSearch.previous")} onClick={() => onMove(-1)}>
        <CaretUp size="var(--icon-xs)" />
      </button>
      <button type="button" className="cfs-action" aria-label={t("conversationSearch.next")} onClick={() => onMove(1)}>
        <CaretDown size="var(--icon-xs)" />
      </button>
      <button type="button" className="cfs-action" aria-label={t("a11y.close")} onClick={onClose}>
        <X size="var(--icon-xs)" />
      </button>
    </div>
  );
}
