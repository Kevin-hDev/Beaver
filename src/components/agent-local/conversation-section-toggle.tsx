import type { KeyboardEvent, ReactNode } from "react";

interface SectionAction {
  /* Nommée à voix haute : le bouton ne porte qu'un dessin. */
  label: string;
  icon: ReactNode;
  onClick: () => void;
}

interface ConversationSectionToggleProps {
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
  action?: SectionAction;
}

/* Titre d'une section de la barre latérale. Le repli et l'action éventuelle
   sont deux commandes voisines, jamais imbriquées : un bouton dans un bouton
   n'est pas du HTML valide et le clavier n'en atteint plus qu'une. */
export function ConversationSectionToggle({ open, onToggle, children, action }: ConversationSectionToggleProps) {
  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onToggle();
    }
  };

  return (
    <div className="conv-section-label">
      <div
        className="conv-section-toggle"
        role="button"
        tabIndex={0}
        aria-expanded={open}
        onClick={onToggle}
        onKeyDown={handleKeyDown}
      >
        {children}
      </div>
      {action && (
        <button
          type="button"
          className="conv-section-action-btn"
          aria-label={action.label}
          title={action.label}
          onClick={action.onClick}
        >
          {action.icon}
        </button>
      )}
    </div>
  );
}
