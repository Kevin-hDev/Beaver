import "./scroll-bottom-button.css";

interface ScrollBottomButtonProps {
  onClick: () => void;
}

/** Retour au bas de la conversation. Flotte au-dessus du champ de saisie :
 *  il apparaît et disparaît au fil du défilement, et rien de ce qui l'entoure
 *  ne doit bouger quand il le fait. */
export function ScrollBottomButton({ onClick }: ScrollBottomButtonProps) {
  return (
    <button type="button" className="icon-btn scroll-bottom-btn" onClick={onClick}>
      <svg className="scroll-bottom-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M12 17V3" />
        <path d="m6 11 6 6 6-6" />
        <path d="M19 21H5" />
      </svg>
    </button>
  );
}
