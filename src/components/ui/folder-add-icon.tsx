import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Dossier marqué d'un « + » : l'action ajoute un dossier au projet, elle n'ouvre
   ni ne parcourt celui qui est déjà là. Le dessin reprend le dossier fermé de
   [FolderStateIcon] pour que la ligne se lise comme les autres de la liste. */
export function FolderAddIcon({ size = "var(--icon-sm)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 21 21">
      <path
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.2"
        d="M3.5 5.5v9a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.497a2 2 0 0 0-2-1.999l-5 .002l-2-2h-4a1 1 0 0 0-1 1m5 6h4m-2 2.056V9.5"
      />
    </InlineIcon>
  );
}
