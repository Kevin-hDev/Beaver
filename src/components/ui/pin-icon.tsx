import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Punaise : l'action « Épingler » du menu d'une conversation, et la même pour
   « Désépingler » — c'est le libellé qui change de sens, pas le dessin.

   Dessin « pinned » de Codicons, par Microsoft, sous licence CC BY 4.0 —
   l'attribution complète est dans THIRD_PARTY_NOTICES.md, seul endroit du
   dépôt qui fait foi pour les licences des dessins repris. */
export function PinIcon({ size = "var(--session-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 16 16">
      <path fill="currentColor" d="M10.059 2.445a1.5 1.5 0 0 0-2.386.353l-2.02 3.79l-2.811.938a.5.5 0 0 0-.196.828L4.793 10.5l-2.647 2.646L2 14l.854-.146L5.5 11.207l2.146 2.147a.5.5 0 0 0 .828-.196l.937-2.811l3.779-2.023a1.5 1.5 0 0 0 .354-2.38zm-1.504.824a.5.5 0 0 1 .796-.118l3.485 3.498a.5.5 0 0 1-.118.794L8.764 9.559a.5.5 0 0 0-.238.283l-.744 2.232L3.926 8.22l2.232-.745a.5.5 0 0 0 .283-.239z" />
    </InlineIcon>
  );
}
