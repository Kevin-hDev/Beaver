import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Actions posées sous un message de la conversation : relancer la réponse,
   modifier son propre message, copier le texte. */

export function ReloadIcon({ size = "var(--icon-sm)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2">
        <path d="M19.933 13.041a8 8 0 1 1-9.925-8.788c3.899-1 7.935 1.007 9.425 4.747" />
        <path d="M20 4v5h-5" />
      </g>
    </InlineIcon>
  );
}

export function EditMessageIcon({ size = "var(--icon-sm)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M8.172 19.828L19.828 8.172c.546-.546.818-.818.964-1.112a2 2 0 0 0 0-1.776c-.146-.295-.418-.567-.964-1.112c-.545-.546-.817-.818-1.112-.964a2 2 0 0 0-1.776 0c-.294.146-.566.418-1.112.964L4.172 15.828c-.579.578-.868.867-1.02 1.235C3 17.43 3 17.839 3 18.657V21h2.343c.818 0 1.226 0 1.594-.152c.367-.152.656-.442 1.235-1.02M12 21h6M14.5 5.5l4 4" />
    </InlineIcon>
  );
}

export function CopyMessageIcon({ size = "var(--icon-sm)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <g fill="currentColor" fillRule="evenodd" clipRule="evenodd">
        <path d="M2 11.667a3.4 3.4 0 0 1 3.4-3.4h2.205v2H5.4a1.4 1.4 0 0 0-1.4 1.4v7.2a1.4 1.4 0 0 0 1.4 1.4h7.2a1.4 1.4 0 0 0 1.4-1.4v-1.8h2v1.8a3.4 3.4 0 0 1-3.4 3.4H5.4a3.4 3.4 0 0 1-3.4-3.4z" />
        <path d="M10 3h8a4 4 0 0 1 4 4v8a4 4 0 0 1-4 4h-8a4 4 0 0 1-4-4V7a4 4 0 0 1 4-4m0 2a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2z" />
      </g>
    </InlineIcon>
  );
}
