import type { KeyboardEvent, ReactNode } from "react";
import { CaretRight } from "@/components/ui/icons";

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
      onClick={onToggle}
      onKeyDown={handleKeyDown}
    >
      <CaretRight
        size="var(--icon-xs)"
        className={`conv-collapse-chevron${open ? " conv-collapse-open" : ""}`}
      />
      {children}
    </div>
  );
}
