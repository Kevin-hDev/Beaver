import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Deux des quatre provenances possibles d'une extension, dans la fenêtre
   d'ajout. Le dossier vient de FolderStateIcon — le même dessin que dans la
   barre latérale, pour que le geste « choisir un dossier » et le dossier qu'on
   voit ensuite dans les projets ne soient pas deux objets différents — et le
   dépôt Git de Phosphor.

   Provenance et licence de chaque tracé dans THIRD_PARTY_NOTICES.md, qui fait
   seul autorité là-dessus. */

export function FileUploadIcon({ size, className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <g
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.5"
      >
        <path d="M4 12v2.544c0 3.245 0 4.868.886 5.967a4 4 0 0 0 .603.603C6.59 22 8.211 22 11.456 22c.705 0 1.058 0 1.381-.114q.1-.036.197-.082c.31-.148.559-.397 1.058-.896l4.736-4.736c.579-.578.867-.867 1.02-1.235c.152-.368.152-.776.152-1.594V10c0-3.771 0-5.657-1.172-6.828S15.771 2 12 2m1 19.5V21c0-2.828 0-4.243.879-5.121C14.757 15 16.172 15 19 15h.5" />
        <path d="M10 5c-.59-.607-2.16-3-3-3S4.59 4.393 4 5m3-2v7" />
      </g>
    </InlineIcon>
  );
}

export function NpmIcon({ size, className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" strokeLinejoin="round" strokeWidth="1.5">
        <path d="M2.5 12c0-4.478 0-6.717 1.391-8.109c1.391-1.39 3.63-1.39 8.109-1.39c4.478 0 6.718 0 8.109 1.39c1.391 1.392 1.391 3.63 1.391 8.11c0 4.478 0 6.717-1.391 8.108S16.479 21.5 12 21.5c-4.478 0-6.718 0-8.109-1.391S2.5 16.479 2.5 12Z" />
        <path strokeLinecap="round" d="M8 7h8a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1h-1.5V9.5H12V17H8a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1" />
      </g>
    </InlineIcon>
  );
}
