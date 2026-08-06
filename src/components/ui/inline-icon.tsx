import type { ReactNode } from "react";
import { svgSizeProps } from "./icon-size";

export interface InlineIconProps {
  size?: number | string;
  className?: string;
}

interface InlineIconFrameProps extends InlineIconProps {
  viewBox: string;
  children: ReactNode;
}

/* Cadre commun aux dessins posés dans la page plutôt que chargés en image :
   `currentColor` ne suit la couleur du texte qui les entoure — et donc l'état
   de la ligne et le thème — que si le tracé appartient au document. Le passer
   par un cadre unique évite qu'un dessin oublie le cadrage ou les attributs
   qui le retirent de l'arbre d'accessibilité. */
export function InlineIcon({ size, className, viewBox, children }: InlineIconFrameProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      {...svgSizeProps(size)}
      viewBox={viewBox}
      className={className}
      aria-hidden="true"
      focusable="false"
    >
      {children}
    </svg>
  );
}
