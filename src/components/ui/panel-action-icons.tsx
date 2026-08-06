import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Actions posées sur l'en-tête d'un panneau ou d'une vue : sortir le contenu
   ailleurs, l'agrandir, le fermer, ouvrir sa documentation. */

export function OpenExternalIcon({ size = "var(--icon-md)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 12 12">
      <path fill="currentColor" d="M4 3a1 1 0 0 0-1 1v4a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1V7a.5.5 0 0 1 1 0v1a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h1a.5.5 0 0 1 0 1zm3 0a.5.5 0 0 1 0-1h2.5a.5.5 0 0 1 .5.5V5a.5.5 0 0 1-1 0V3.707L7.354 5.354a.5.5 0 1 1-.708-.708L8.293 3z" />
    </InlineIcon>
  );
}

export function FullscreenIcon({ size = "var(--icon-md)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 16 16">
      <path fill="currentColor" d="M13 3.5v3a.5.5 0 0 1-1 0V4.71L9.85 6.86a.5.5 0 0 1-.707-.707l2.15-2.15h-1.79a.5.5 0 0 1 0-1h3a.5.5 0 0 1 .351.144l.004.004a.5.5 0 0 1 .144.348v.004zM3.5 9a.5.5 0 0 1 .5.5v1.79l2.15-2.15a.5.5 0 0 1 .707.707l-2.15 2.15h1.79a.5.5 0 0 1 0 1H3.494a.5.5 0 0 1-.497-.499v-3a.5.5 0 0 1 .5-.5z" />
      <path fill="currentColor" fillRule="evenodd" clipRule="evenodd" d="M0 4.8c0-1.68 0-2.52.327-3.16A3.02 3.02 0 0 1 1.637.33c.642-.327 1.48-.327 3.16-.327h6.4c1.68 0 2.52 0 3.16.327a3.02 3.02 0 0 1 1.31 1.31c.327.642.327 1.48.327 3.16v6.4c0 1.68 0 2.52-.327 3.16a3 3 0 0 1-1.31 1.31c-.642.327-1.48.327-3.16.327h-6.4c-1.68 0-2.52 0-3.16-.327a3 3 0 0 1-1.31-1.31C0 13.718 0 12.88 0 11.2zM4.8 1h6.4c.857 0 1.44 0 1.89.038c.438.035.663.1.819.18c.376.192.682.498.874.874c.08.156.145.38.18.819c.037.45.038 1.03.038 1.89v6.4c0 .857-.001 1.44-.038 1.89c-.036.438-.101.663-.18.819a2 2 0 0 1-.874.874c-.156.08-.381.145-.819.18c-.45.036-1.03.037-1.89.037H4.8c-.857 0-1.44 0-1.89-.037c-.438-.036-.663-.101-.819-.18a2 2 0 0 1-.874-.874c-.08-.156-.145-.381-.18-.82C1 12.64.999 12.06.999 11.2V4.8c0-.856.001-1.44.038-1.89c.036-.437.101-.662.18-.818c.192-.376.498-.682.874-.874c.156-.08.381-.145.819-.18C3.36 1 3.94 1 4.8 1" />
    </InlineIcon>
  );
}

export function CloseIcon({ size = "var(--panel-close-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2.5" d="m7 7l10 10M7 17L17 7" />
    </InlineIcon>
  );
}

export function DocumentationIcon({ size = "var(--chrome-icon-docs)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="currentColor" d="M13 21q-.425 0-.712-.288T12 20t.288-.712T13 19h2.825l-2.3-1.625q-.35-.25-.413-.638t.163-.737t.625-.413t.75.163l2.325 1.6L16 14.7q-.15-.4.025-.762t.575-.513t.775.025t.525.575l.95 2.65l.75-2.725q.125-.4.462-.612t.738-.088t.625.463t.1.737l-1.55 5.8q-.1.35-.363.55T19 21zm-9-3q-.425 0-.712-.288T3 17t.288-.712T4 16h6.075q-.075.525-.062 1t.087 1zm0-4q-.425 0-.712-.288T3 13t.288-.712T4 12h8.65q-.575.4-1.037.9T10.8 14zm0-4q-.425 0-.712-.288T3 9t.288-.712T4 8h13q.425 0 .713.288T18 9t-.288.713T17 10zm0-4q-.425 0-.712-.288T3 5t.288-.712T4 4h13q.425 0 .713.288T18 5t-.288.713T17 6z" />
    </InlineIcon>
  );
}
