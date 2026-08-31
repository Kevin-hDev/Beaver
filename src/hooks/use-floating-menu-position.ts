import { useCallback, useLayoutEffect, useRef, useState, type CSSProperties } from "react";

type FloatingAlign = "left" | "right" | "before" | "after";
type FloatingPlacement = "above" | "below" | "auto";

const VIEWPORT_PADDING = 12;
const HIDDEN_STYLE: CSSProperties = {
  position: "fixed",
  top: 0,
  left: 0,
  right: "auto",
  bottom: "auto",
  visibility: "hidden",
  zIndex: 1000,
};

/* spanRef : élément dont le menu reprend le bord gauche et la largeur, quand
   ceux de l'ancre ne conviennent pas — un menu ouvert depuis un petit bouton
   mais qui doit couvrir la zone à laquelle il appartient. Le placement vertical
   reste celui de l'ancre. */
export function useFloatingMenuPosition(
  open: boolean,
  align: FloatingAlign = "left",
  gap = 4,
  placement: FloatingPlacement = "above",
  matchAnchorWidth = false,
  spanRef?: React.RefObject<HTMLElement | null>,
  horizontalRef?: React.RefObject<HTMLElement | null>,
) {
  const anchorRef = useRef<HTMLElement | null>(null);
  const floatingRef = useRef<HTMLDivElement | null>(null);
  const resolvedPlacementRef = useRef<"above" | "below" | null>(null);
  const [style, setStyle] = useState<CSSProperties>(HIDDEN_STYLE);

  const update = useCallback(() => {
    const anchor = anchorRef.current;
    const floating = floatingRef.current;
    if (!open || !anchor || !floating) return;

    const anchorRect = anchor.getBoundingClientRect();
    const spanRect = spanRef?.current?.getBoundingClientRect() ?? null;
    const width = Math.max(
      spanRect ? spanRect.width : floating.offsetWidth,
      matchAnchorWidth ? anchorRect.width : 0,
    );
    const height = floating.offsetHeight;
    const maxWidth = Math.max(0, window.innerWidth - (VIEWPORT_PADDING * 2));
    const boundedWidth = Math.min(width, maxWidth);
    const maxLeft = Math.max(VIEWPORT_PADDING, window.innerWidth - boundedWidth - VIEWPORT_PADDING);
    const horizontalRect = horizontalRef?.current?.getBoundingClientRect() ?? spanRect ?? anchorRect;
    let rawLeft = align === "right"
      ? horizontalRect.right - width
      : align === "before"
        ? horizontalRect.left - width - gap
        : align === "after"
          ? horizontalRect.right + gap
          : horizontalRect.left;
    if (align === "before" && rawLeft < VIEWPORT_PADDING) {
      const after = horizontalRect.right + gap;
      if (after + boundedWidth <= window.innerWidth - VIEWPORT_PADDING) rawLeft = after;
    }
    const left = Math.min(Math.max(rawLeft, VIEWPORT_PADDING), maxLeft);
    const availableAbove = Math.max(0, anchorRect.top - gap - VIEWPORT_PADDING);
    const availableBelow = Math.max(
      0,
      window.innerHeight - anchorRect.bottom - gap - VIEWPORT_PADDING,
    );
    if (!resolvedPlacementRef.current) {
      resolvedPlacementRef.current = placement === "auto"
        ? height > availableAbove && availableBelow > availableAbove
          ? "below"
          : "above"
        : placement;
    }
    const opensBelow = resolvedPlacementRef.current === "below";
    const maxHeight = opensBelow ? availableBelow : availableAbove;

    setStyle({
      position: "fixed",
      top: opensBelow ? anchorRect.bottom + gap : "auto",
      left,
      maxWidth,
      maxHeight,
      width: spanRect ? boundedWidth : undefined,
      minWidth: matchAnchorWidth ? anchorRect.width : undefined,
      right: "auto",
      bottom: opensBelow ? "auto" : window.innerHeight - anchorRect.top + gap,
      visibility: "visible",
      zIndex: 1000,
    });
  }, [align, gap, horizontalRef, matchAnchorWidth, open, placement, spanRef]);

  useLayoutEffect(() => {
    if (!open) {
      resolvedPlacementRef.current = null;
      return;
    }

    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [open, update]);

  return { anchorRef, floatingRef, floatingStyle: style, updateFloatingPosition: update };
}

export function floatingMenuPortalRoot(): HTMLElement {
  return document.querySelector<HTMLElement>(".app-root") ?? document.body;
}
