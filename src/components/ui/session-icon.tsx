import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Dessin unique d'une discussion, partout où l'application en désigne une :
   l'onglet du rail, la recherche, les chats archivés. Il vit ici et non avec
   les icônes du rail parce que trois domaines s'en servent. */
export function SessionIcon({ size = "var(--nav-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" d="M12 11v-.5m4 .5v-.5M8 11v-.5m-4.536 6.328C2 15.657 2 14.771 2 11s0-5.657 1.464-6.828C4.93 3 7.286 3 12 3s7.071 0 8.535 1.172S22 7.229 22 11s0 4.657-1.465 5.828C19.072 18 16.714 18 12 18c-2.51 0-3.8 1.738-6 3v-3.212c-1.094-.163-1.899-.45-2.536-.96" />
    </InlineIcon>
  );
}
