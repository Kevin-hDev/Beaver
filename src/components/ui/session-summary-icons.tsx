import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Les six rangées de la bulle « Résumé de conversation ». La branche emprunte
   son dessin à Phosphor, les cinq autres sont ici. Tous prennent leur taille de
   --summary-row-icon-size, qui fait seul autorité sur ce panneau, et leur
   couleur du texte qui les entoure.

   Provenance et licence de chaque tracé dans THIRD_PARTY_NOTICES.md, qui fait
   seul autorité là-dessus. */

export function CommitIcon({ size = "var(--summary-row-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="currentColor" d="M8.813 15.863Q7.45 14.725 7.1 13H3q-.425 0-.712-.288T2 12t.288-.712T3 11h4.1q.35-1.725 1.713-2.863T12 7t3.188 1.138T16.9 11H21q.425 0 .713.288T22 12t-.288.713T21 13h-4.1q-.35 1.725-1.712 2.863T12 17t-3.187-1.137M12 15q1.25 0 2.125-.875T15 12t-.875-2.125T12 9t-2.125.875T9 12t.875 2.125T12 15" />
    </InlineIcon>
  );
}

export function ModificationIcon({ size = "var(--summary-row-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" strokeLinejoin="round" strokeWidth="1.5">
        <path d="M13 20.827V22h1.173c.41 0 .614 0 .799-.076c.184-.076.328-.221.618-.51l4.823-4.825c.273-.273.41-.41.483-.556c.139-.28.139-.61 0-.89c-.073-.147-.21-.283-.483-.556s-.41-.41-.556-.483a1 1 0 0 0-.89 0c-.147.073-.284.21-.557.483l-4.823 4.824c-.29.289-.434.434-.51.618s-.077.388-.077.798Z" />
        <path strokeLinecap="round" d="M19 11s0-1.57-.152-1.937s-.441-.657-1.02-1.235l-4.736-4.736c-.499-.499-.748-.748-1.058-.896a2 2 0 0 0-.197-.082C11.514 2 11.161 2 10.456 2c-3.245 0-4.868 0-5.967.886a4 4 0 0 0-.603.603C3 4.59 3 6.211 3 9.456V14c0 3.771 0 5.657 1.172 6.828C5.235 21.892 6.886 21.99 10 22m2-19.5V3c0 2.828 0 4.243.879 5.121C13.757 9 15.172 9 18 9h.5" />
      </g>
    </InlineIcon>
  );
}

export function PlanIcon({ size = "var(--summary-row-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5">
        <path d="M13.5 2h-5a1.5 1.5 0 1 0 0 3h5a1.5 1.5 0 0 0 0-3M7 15h3.429M7 11h8" />
        <path d="M19 12V9.483c0-2.829 0-4.243-.879-5.122c-.641-.64-1.567-.814-3.121-.861M11 22H9c-2.828 0-4.243 0-5.121-.88C3 20.243 3 18.829 3 16V9.482c0-2.829 0-4.243.879-5.122c.641-.64 1.568-.814 3.12-.861m8.738 18.154L14 22l.347-1.737c.07-.352.244-.676.499-.93l4.065-4.066a.91.91 0 0 1 1.288 0l.534.534a.91.91 0 0 1 0 1.288l-4.065 4.065a1.8 1.8 0 0 1-.931.499" />
      </g>
    </InlineIcon>
  );
}

/* Visage rond au trait, yeux pleins. Les yeux restent pleins alors que le reste
   est au trait : la rangée affiche son dessin sous les 16 px de l'icône
   courante, taille à laquelle l'intérieur d'un œil évidé mesure moins d'un
   pixel et se rend en tache grise. */
export function SubagentSummaryIcon({ size = "var(--summary-row-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <circle cx="12" cy="12" r="9.5" fill="none" stroke="currentColor" strokeWidth="2" />
      <rect x="7.8" y="7.4" width="3" height="5.2" rx="1.5" fill="currentColor" />
      <rect x="13.2" y="7.4" width="3" height="5.2" rx="1.5" fill="currentColor" />
    </InlineIcon>
  );
}

export function TodoListIcon({ size = "var(--summary-row-icon-size)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 640 640">
      <path fill="currentColor" d="M197.8 100.3c10.9 7.6 13.5 22.6 5.9 33.4l-56 80c-4.1 5.8-10.5 9.5-17.6 10.1S116 222 111 217l-40-40c-9.3-9.4-9.3-24.6 0-34s24.6-9.3 34 0l19.8 19.8l39.6-56.6c7.6-10.9 22.6-13.5 33.4-5.9m0 160c10.9 7.6 13.5 22.6 5.9 33.4l-56 80c-4.1 5.8-10.5 9.5-17.6 10.1S116 382 111 377l-40-40c-9.4-9.4-9.4-24.6 0-33.9s24.6-9.4 33.9 0l19.8 19.8l39.6-56.6c7.6-10.9 22.6-13.5 33.4-5.9zM288 160c0-17.7 14.3-32 32-32h224c17.7 0 32 14.3 32 32s-14.3 32-32 32H320c-17.7 0-32-14.3-32-32m0 160c0-17.7 14.3-32 32-32h224c17.7 0 32 14.3 32 32s-14.3 32-32 32H320c-17.7 0-32-14.3-32-32m-64 160c0-17.7 14.3-32 32-32h288c17.7 0 32 14.3 32 32s-14.3 32-32 32H256c-17.7 0-32-14.3-32-32m-96-40c22.1 0 40 17.9 40 40s-17.9 40-40 40s-40-17.9-40-40s17.9-40 40-40" />
    </InlineIcon>
  );
}
