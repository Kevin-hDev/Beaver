import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Dessin unique de « copier dans le presse-papiers ». Il vit à part et non avec
   les actions d'un message parce que trois domaines s'en servent : les boutons
   sous un message, un bloc de code, et le menu d'une conversation. */
export function CopyIcon({ size = "var(--icon-sm)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <g fill="currentColor" fillRule="evenodd" clipRule="evenodd">
        <path d="M2 11.667a3.4 3.4 0 0 1 3.4-3.4h2.205v2H5.4a1.4 1.4 0 0 0-1.4 1.4v7.2a1.4 1.4 0 0 0 1.4 1.4h7.2a1.4 1.4 0 0 0 1.4-1.4v-1.8h2v1.8a3.4 3.4 0 0 1-3.4 3.4H5.4a3.4 3.4 0 0 1-3.4-3.4z" />
        <path d="M10 3h8a4 4 0 0 1 4 4v8a4 4 0 0 1-4 4h-8a4 4 0 0 1-4-4V7a4 4 0 0 1 4-4m0 2a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2z" />
      </g>
    </InlineIcon>
  );
}
