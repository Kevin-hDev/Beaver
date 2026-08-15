import { InlineIcon } from "@/components/ui/inline-icon";

/* Les trois visages du bouton au bout du champ de saisie. Ils partagent le même
   cadre arrondi et ne diffèrent que par leur centre : d'un état à l'autre, seul
   le symbole intérieur change, et le bouton ne paraît pas remplacé.

   La taille vient de la classe, pas d'une prop : les trois se règlent ensemble
   par [--chat-send-icon-size]. */

const FRAME = "M9 22h6c5 0 7-2 7-7V9c0-5-2-7-7-7H9C4 2 2 4 2 9v6c0 5 2 7 7 7";

function Frame({ children }: { children?: React.ReactNode }) {
  return (
    <InlineIcon viewBox="0 0 24 24" className="send-stop-icon">
      <g fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5">
        <path d={FRAME} />
        {children}
      </g>
    </InlineIcon>
  );
}

export function SendIcon() {
  return <Frame><path d="m9 9.51l3-3l3 3m-3-3v8m-6 2c3.89 1.3 8.11 1.3 12 0" /></Frame>;
}

export function StopIcon() {
  return <Frame><rect x="7.5" y="7.5" width="9" height="9" rx="2" fill="currentColor" stroke="none" /></Frame>;
}

/* Les lettres sont tracées, non écrites : le dessin précédent posait un vrai
   texte dans le SVG, que le rendu de la police pouvait décaler ou rogner. */
export function ConfirmStopIcon() {
  return (
    <Frame>
      <g strokeWidth="1.1">
        <path d="M8.3 9.6H5.6v4.8h2.7M5.6 12h2.2" />
        <path d="M12.75 10.26A1.6 1.225 0 1 0 11.3 12a1.6 1.225 0 1 1-1.45 1.74" />
        <path d="M17.65 10.12A2.1 2.45 0 1 0 17.65 13.88" />
      </g>
    </Frame>
  );
}
