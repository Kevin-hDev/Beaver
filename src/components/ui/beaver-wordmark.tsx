import { useId } from "react";

import {
  WORDMARK_BODY_PATH,
  WORDMARK_BRICKS_PATH,
  WORDMARK_VIEWBOX,
} from "./beaver-wordmark-paths";
import { BRAND } from "@/lib/brand";
import { cn } from "@/lib/utils";
import "./beaver-wordmark.css";

interface BeaverWordmarkProps {
  /** Classes de l'appelant : c'est lui qui fixe la largeur, jamais la primitive. */
  className?: string;
  /** Le nom reste lisible pour un lecteur d'écran, que le mot soit dessiné ou écrit. */
  title?: string;
}

/**
 * Le logotype « BEAVER. » — autorité unique du wordmark dans l'application.
 * Tout écran qui affiche le nom en grand passe par ici.
 *
 * L'identifiant du dégradé est propre à chaque instance : deux logotypes sur un
 * même écran partageraient sinon le même identifiant, et le document porterait
 * deux définitions concurrentes sous un seul nom.
 */
export function BeaverWordmark({ className, title = BRAND.displayName }: BeaverWordmarkProps) {
  const inkId = `bw-ink-${useId()}`;

  return (
    <svg
      className={cn("bw-mark", className)}
      viewBox={WORDMARK_VIEWBOX}
      role="img"
      aria-label={title}
    >
      <defs>
        <linearGradient id={inkId} x1="0" y1="0" x2="0" y2="1">
          <stop className="bw-ink-top" offset="0" />
          <stop className="bw-ink-mid" offset="0.7" />
          <stop className="bw-ink-bottom" offset="1" />
        </linearGradient>
      </defs>
      <path className="bw-body" fill={`url(#${inkId})`} fillRule="evenodd" d={WORDMARK_BODY_PATH} />
      <path className="bw-bricks" fillRule="evenodd" d={WORDMARK_BRICKS_PATH} />
    </svg>
  );
}
