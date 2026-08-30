import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Cadran devant le titre des réveils. Une horloge dit l'heure programmée, là où
   le battement de cœur qu'elle remplace disait la surveillance — ce que la page
   ne fait pas.

   Dessin « clock-hour-8 » de Tabler Icons, sous licence MIT — l'attribution
   complète est dans THIRD_PARTY_NOTICES.md, seul endroit du dépôt qui fait foi
   pour les licences des dessins repris. */
export function WakeupClockIcon({ size = "var(--icon-xl)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="2"
        d="M3 12a9 9 0 1 0 18 0a9 9 0 1 0-18 0m9 0l-3 2m3-7v5"
      />
    </InlineIcon>
  );
}
