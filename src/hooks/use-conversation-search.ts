import { useCallback, useEffect, useMemo, useState } from "react";
import { matchesAppShortcut } from "@/lib/app-shortcuts";
import { isCompressionContextOnlyMessage, isCompressionSummaryMessage } from "@/lib/context-messages";
import type { AgentMessage } from "@/types/agent";

const MAX_QUERY_LENGTH = 120;
// La session est déjà bornée, mais le résultat l’est aussi pour garder une
// navigation instantanée même dans une conversation exceptionnellement longue.
const MAX_MATCHES = 200;

export function useConversationSearch(messages: AgentMessage[]) {
  const [open, setOpen] = useState(false);
  const [query, setRawQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const [focusRequest, setFocusRequest] = useState(0);
  const matches = useMemo(() => findMatches(messages, query), [messages, query]);
  const safeIndex = matches.length > 0 ? activeIndex % matches.length : 0;
  const activeMessageId = matches[safeIndex] ?? null;
  const close = useCallback(() => setOpen(false), []);
  const setQuery = useCallback((value: string) => {
    setRawQuery(value.slice(0, MAX_QUERY_LENGTH));
    setActiveIndex(0);
  }, []);
  const move = useCallback((direction: 1 | -1) => {
    if (matches.length < 1) return;
    setActiveIndex((current) => (current + direction + matches.length) % matches.length);
  }, [matches.length]);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (matchesAppShortcut(event, "searchConversation")) {
        event.preventDefault();
        setOpen(true);
        setFocusRequest((current) => current + 1);
        return;
      }
      if (open && matchesAppShortcut(event, "cancelEdit")) {
        event.preventDefault();
        event.stopImmediatePropagation();
        close();
      }
    };
    window.addEventListener("keydown", handler, true);
    return () => window.removeEventListener("keydown", handler, true);
  }, [close, open]);

  useEffect(() => {
    if (!open || !activeMessageId) return;
    const targets = document.querySelectorAll<HTMLElement>("[data-message-id]");
    const target = Array.from(targets).find((element) => (
      element.dataset.messageId === activeMessageId
    ));
    target?.scrollIntoView({ block: "center", behavior: "smooth" });
  }, [activeMessageId, open]);

  return {
    open,
    close,
    query,
    setQuery,
    move,
    activeMessageId,
    activePosition: matches.length > 0 ? safeIndex + 1 : 0,
    totalMatches: matches.length,
    focusRequest,
  };
}

function findMatches(messages: AgentMessage[], query: string): string[] {
  const normalized = query.trim().toLocaleLowerCase();
  if (!normalized) return [];
  const matches: string[] = [];
  for (const message of messages) {
    if (message.role === "tool") continue;
    if (isCompressionContextOnlyMessage(message) || isCompressionSummaryMessage(message)) continue;
    const text = `${message.content}\n${message.thinking ?? ""}`.toLocaleLowerCase();
    if (text.includes(normalized)) matches.push(message.id);
    if (matches.length >= MAX_MATCHES) break;
  }
  return matches;
}
