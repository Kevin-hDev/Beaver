import type { ReactNode } from "react";

interface ConversationEmptyNoteProps {
  children: ReactNode;
  /* Aligne la note sous les conversations d'un projet plutôt que sous son titre. */
  indented?: boolean;
}

/* Ce que montre une section dépliée qui n'a rien à montrer. Sans elle, la
   section se replie sur du vide et on ne sait pas si elle charge ou si elle
   est vide. */
export function ConversationEmptyNote({ children, indented }: ConversationEmptyNoteProps) {
  return (
    <div className={indented ? "conv-empty-note conv-empty-note-indented" : "conv-empty-note"}>
      {children}
    </div>
  );
}
