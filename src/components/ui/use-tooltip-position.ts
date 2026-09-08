import { useCallback, useEffect, useRef, useState, type CSSProperties, type RefObject } from "react";
import { useAppSurfaceActive } from "@/components/layout/app-surface-activity";

/* Écart entre la bulle et l'élément qu'elle décrit. */
const GAP = 6;
/* Marge gardée avec les bords de la fenêtre. */
const EDGE = 8;

export interface TooltipPosition {
  side: "above" | "below";
  style: CSSProperties;
}

interface TooltipPlacement {
  position: TooltipPosition | null;
  bubbleRef: (node: HTMLElement | null) => void;
}

/* Autorité unique du placement d'une infobulle : celui qui la pose n'a pas à
   choisir son côté. Le côté vient de la place réellement disponible, mesurée à
   l'ouverture — sinon chaque appelant doit deviner, et une bulle posée au ras
   d'un bord se perd hors de la vue sans laisser de trace. */
export function useTooltipPosition(
  visible: boolean,
  anchorRef: RefObject<HTMLElement | null>,
  align: "center" | "right",
): TooltipPlacement {
  const surfaceActive = useAppSurfaceActive();
  const [position, setPosition] = useState<TooltipPosition | null>(null);
  const bubble = useRef<HTMLElement | null>(null);

  const place = useCallback(() => {
    const anchor = anchorRef.current;
    const node = bubble.current;
    if (!anchor || !node) return;
    const rect = anchor.getBoundingClientRect();
    const below = rect.bottom + GAP + node.offsetHeight <= window.innerHeight - EDGE;
    setPosition({
      side: below ? "below" : "above",
      style: {
        left: horizontalOffset(rect, node.offsetWidth, align),
        ...(below
          ? { top: rect.bottom + GAP }
          : { bottom: window.innerHeight - rect.top + GAP }),
      },
    });
  }, [align, anchorRef]);

  /* Mesuré au rattachement du nœud, pas dans un effet : la bulle n'a de taille
     qu'une fois montée, et son placement doit précéder sa première peinture. */
  const bubbleRef = useCallback((node: HTMLElement | null) => {
    bubble.current = node;
    if (node) place();
  }, [place]);

  useEffect(() => {
    if (!surfaceActive || !visible) return;
    window.addEventListener("resize", place);
    window.addEventListener("scroll", place, true);
    return () => {
      window.removeEventListener("resize", place);
      window.removeEventListener("scroll", place, true);
    };
  }, [place, surfaceActive, visible]);

  return { position, bubbleRef };
}

/* Le bornage aux bords de la fenêtre passe avant l'alignement demandé : une
   bulle plus large que son élément sort de l'écran en bord de fenêtre. */
function horizontalOffset(rect: DOMRect, width: number, align: "center" | "right"): number {
  const raw = align === "right" ? rect.right - width : rect.left + (rect.width - width) / 2;
  const max = Math.max(EDGE, window.innerWidth - width - EDGE);
  return Math.min(Math.max(raw, EDGE), max);
}
