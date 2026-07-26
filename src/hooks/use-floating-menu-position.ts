import { useCallback, useLayoutEffect, useRef, useState, type CSSProperties } from "react";

type FloatingAlign = "left" | "right";
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

export function useFloatingMenuPosition(
  open: boolean,
  align: FloatingAlign = "left",
  gap = 4,
  placement: FloatingPlacement = "above",
  matchAnchorWidth = false,
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
    const width = Math.max(
      floating.offsetWidth,
      matchAnchorWidth ? anchorRect.width : 0,
    );
    const height = floating.offsetHeight;
    const maxWidth = Math.max(0, window.innerWidth - (VIEWPORT_PADDING * 2));
    const boundedWidth = Math.min(width, maxWidth);
    const maxLeft = Math.max(VIEWPORT_PADDING, window.innerWidth - boundedWidth - VIEWPORT_PADDING);
    const rawLeft = align === "right" ? anchorRect.right - width : anchorRect.left;
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
      minWidth: matchAnchorWidth ? anchorRect.width : undefined,
      right: "auto",
      bottom: opensBelow ? "auto" : window.innerHeight - anchorRect.top + gap,
      visibility: "visible",
      zIndex: 1000,
    });
  }, [align, gap, matchAnchorWidth, open, placement]);

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
