import { useState, useRef, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { floatingMenuPortalRoot, useFloatingMenuPosition } from "@/hooks/use-floating-menu-position";
import "./interactive-choice-tooltip.css";

interface InteractiveChoiceTooltipProps {
  /** Contenu tronqué affiché dans le panneau (label/description). */
  children: ReactNode;
  /** Texte complet à afficher dans la bubble au survol, si troncation détectée. */
  fullText: string;
  /** Classe additionnelle sur le conteneur (ex. positionnement grid). */
  className?: string;
}

/**
 * Affiche `children` normalement. Si le contenu déborde (texte coupé par
 * line-clamp), une bubble affichant `fullText` apparaît au survol.
 */
export function InteractiveChoiceTooltip({ children, fullText, className }: InteractiveChoiceTooltipProps) {
  const hostRef = useRef<HTMLSpanElement | null>(null);
  const [truncated, setTruncated] = useState(false);
  const [open, setOpen] = useState(false);
  const { anchorRef, floatingRef, floatingStyle } = useFloatingMenuPosition(open, "left", 6);

  const handleEnter = () => {
    const host = hostRef.current;
    if (!host) return;
    // Le contenu clamped vit dans un enfant (span). On vérifie le débordement
    // sur le premier enfant non-texte, sinon sur le host lui-même.
    const target = host.firstElementChild instanceof HTMLElement
      ? host.firstElementChild
      : host;
    const isTruncated = target.scrollWidth > target.clientWidth + 1
      || target.scrollHeight > target.clientHeight + 1;
    if (isTruncated) {
      setTruncated(true);
      setOpen(true);
    }
  };

  const handleLeave = () => setOpen(false);

  const setHostRef = (node: HTMLSpanElement | null) => {
    hostRef.current = node;
    anchorRef.current = node;
  };

  const hostClass = className ? `icp-tt-host ${className}` : "icp-tt-host";

  return (
    <>
      <span
        ref={setHostRef}
        className={hostClass}
        onMouseEnter={handleEnter}
        onMouseLeave={handleLeave}
      >
        {children}
      </span>
      {truncated && open && createPortal(
        <div ref={floatingRef} className="icp-tt-bubble" style={floatingStyle} role="tooltip">
          {fullText}
        </div>,
        floatingMenuPortalRoot(),
      )}
    </>
  );
}
