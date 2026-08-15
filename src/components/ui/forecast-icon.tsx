import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Barres de hauteurs croissantes : la prévision. Le dessin vit ici et non avec
   les onglets des réglages parce que la conversation le montre aussi, sur les
   lignes des outils de prévision. */
export function ForecastIcon({ size = "var(--nav-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="currentColor" d="M20 13.75a.75.75 0 0 0-.75-.75h-3a.75.75 0 0 0-.75.75v6.75H14V4.25c0-.728-.002-1.2-.048-1.546c-.044-.325-.115-.427-.172-.484s-.159-.128-.484-.172C12.949 2.002 12.478 2 11.75 2s-1.2.002-1.546.048c-.325.044-.427.115-.484.172s-.128.159-.172.484c-.046.347-.048.818-.048 1.546V20.5H8V8.75A.75.75 0 0 0 7.25 8h-3a.75.75 0 0 0-.75.75V20.5H1.75a.75.75 0 0 0 0 1.5h20a.75.75 0 0 0 0-1.5H20z" />
    </InlineIcon>
  );
}
