import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Les trois onglets de la page Extensions. Chacun prend sa taille de
   --extension-tab-icon-size, sauf l'hôte dont la raison est écrite avec lui.

   Ces dessins n'ont pas de variante pleine, contrairement aux icônes Phosphor
   qu'ils remplacent : l'onglet actif se distingue par sa pastille — fond,
   bordure, ombre et couleur d'encre — et non par l'épaisseur de son tracé.

   Provenance et licence de chaque tracé dans THIRD_PARTY_NOTICES.md, qui fait
   seul autorité là-dessus. */

export function PluginsIcon({ size = "var(--extension-tab-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="none" stroke="currentColor" strokeLinecap="round" strokeWidth="1.5" d="M4.513 19.487c2.512 2.392 5.503 1.435 6.7.466c.618-.501.897-.825 1.136-1.065c.837-.777.784-1.555.24-2.177c-.219-.249-1.616-1.591-2.956-2.967c-.694-.694-1.172-1.184-1.582-1.58c-.547-.546-1.026-1.172-1.744-1.154c-.658 0-1.136.58-1.735 1.179c-.688.688-1.196 1.555-1.375 2.333c-.539 2.273.299 3.888 1.316 4.965Zm0 0L2 21.999M19.487 4.515c-2.513-2.394-5.494-1.42-6.69-.45c-.62.502-.898.826-1.138 1.066c-.837.778-.784 1.556-.239 2.178c.078.09.31.32.635.644m7.432-3.438c1.017 1.077 1.866 2.71 1.327 4.985c-.18.778-.688 1.645-1.376 2.334c-.598.598-1.077 1.179-1.735 1.179c-.718.018-1.09-.502-1.639-1.048m3.423-7.45L22 2m-5.936 9.964c-.41-.395-.994-.993-1.688-1.687c-.858-.882-1.74-1.75-2.321-2.325m4.009 4.012l-1.562 1.525m-3.99-3.984l1.543-1.553" />
    </InlineIcon>
  );
}

export function CustomExtensionsIcon({ size = "var(--extension-tab-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 512 512">
      <path fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="32" d="M413.66 246.1H386a2 2 0 0 1-2-2v-77.24A38.86 38.86 0 0 0 345.14 128H267.9a2 2 0 0 1-2-2V98.34c0-27.14-21.5-49.86-48.64-50.33a49.53 49.53 0 0 0-50.4 49.51V126a2 2 0 0 1-2 2H87.62A39.74 39.74 0 0 0 48 167.62V238a2 2 0 0 0 2 2h26.91c29.37 0 53.68 25.48 54.09 54.85c.42 29.87-23.51 57.15-53.29 57.15H50a2 2 0 0 0-2 2v70.38A39.74 39.74 0 0 0 87.62 464H158a2 2 0 0 0 2-2v-20.93c0-30.28 24.75-56.35 55-57.06c30.1-.7 57 20.31 57 50.28V462a2 2 0 0 0 2 2h71.14A38.86 38.86 0 0 0 384 425.14v-78a2 2 0 0 1 2-2h28.48c27.63 0 49.52-22.67 49.52-50.4s-23.2-48.64-50.34-48.64" />
    </InlineIcon>
  );
}

/* Le serveur garde la taille courante quand ses deux voisins descendent d'un
   quart : son tracé est large et plat, il n'occupe que la moitié de la hauteur
   de son cadre, et réduit il devenait le petit de la rangée. */
export function ExtensionHostIcon({ size = "var(--icon-md)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" d="M5.25 14.25h13.5m-13.5 0a3 3 0 0 1-3-3m3 3a3 3 0 1 0 0 6h13.5a3 3 0 1 0 0-6m-16.5-3a3 3 0 0 1 3-3h13.5a3 3 0 0 1 3 3m-19.5 0a4.5 4.5 0 0 1 .9-2.7L5.738 5.1a3.38 3.38 0 0 1 2.7-1.35h7.124c1.063 0 2.063.5 2.7 1.35l2.588 3.45a4.5 4.5 0 0 1 .9 2.7m0 0a3 3 0 0 1-3 3m0 3h.008v.008h-.008zm0-6h.008v.008h-.008zm-3 6h.008v.008h-.008zm0-6h.008v.008h-.008z" />
    </InlineIcon>
  );
}
