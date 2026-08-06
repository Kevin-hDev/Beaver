import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Un seul dessin pour l'onglet « Conversations archivées » des Réglages et
   pour l'action qui y envoie une session : ce sont les deux bouts du même
   geste, et deux signes différents feraient croire à deux destinations.
   La taille par défaut est celle des menus de la barre latérale, d'où l'action
   est lancée ; les Réglages, qui affichent leurs dessins plus grands, passent
   la leur. */
export function ArchiveBoxIcon({ size = "var(--session-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 25">
      <path fill="currentColor" d="M10 11.565a.75.75 0 1 0 0 1.5h4a.75.75 0 0 0 0-1.5z" />
      <path fill="currentColor" d="M2 6.064a2.25 2.25 0 0 1 2.25-2.25h15.5A2.25 2.25 0 0 1 22 6.064v1a2.25 2.25 0 0 1-1.25 2.017v9.984a2.25 2.25 0 0 1-2.25 2.25h-13a2.25 2.25 0 0 1-2.25-2.25V9.08A2.25 2.25 0 0 1 2 7.064zm2.75 3.25v9.75c0 .415.336.75.75.75h13a.75.75 0 0 0 .75-.75v-9.75zm15.75-2.25v-1a.75.75 0 0 0-.75-.75H4.25a.75.75 0 0 0-.75.75v1c0 .415.336.75.75.75h15.5a.75.75 0 0 0 .75-.75" />
    </InlineIcon>
  );
}
