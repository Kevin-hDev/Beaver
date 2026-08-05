import type { KeyboardEvent, ReactNode } from "react";

interface ConversationSectionToggleProps {
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
}

export function ConversationSectionToggle({ open, onToggle, children }: ConversationSectionToggleProps) {
  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onToggle();
    }
  };

  return (
    <div
      className="conv-section-label conv-section-toggle"
      role="button"
      tabIndex={0}
      aria-expanded={open}
      onClick={onToggle}
      onKeyDown={handleKeyDown}
    >
      {children}
    </div>
  );
}
