import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Coche dans une pastille dentelée : une action vient d'aboutir. Licence de la
   source dans THIRD_PARTY_NOTICES.md, qui fait seul autorité là-dessus.

   Le cadre déborde le tracé d'une unité sur chaque bord : le dessin d'origine
   remplit son carré alors que les icônes voisines n'en occupent que quatre
   cinquièmes, et il paraissait plus gros qu'elles à taille demandée égale. */
export function ValidateIcon({ size = "var(--icon-sm)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="-1 -1 26 26">
      <path fill="none" stroke="currentColor" strokeWidth="2" d="M20 15c-1 1 1.25 3.75 0 5s-4-1-5 0s-1.5 3-3 3s-2-2-3-3s-3.75 1.25-5 0s1-4 0-5s-3-1.5-3-3s2-2 3-3s-1.25-3.75 0-5s4 1 5 0s1.5-3 3-3s2 2 3 3s3.75-1.25 5 0s-1 4 0 5s3 1.5 3 3s-2 2-3 3ZM7 12l3 3l7-7" />
    </InlineIcon>
  );
}
