import { useEffect, useLayoutEffect, useRef, useState } from "react";

interface Options {
  isOpen: boolean;
  instantLayout: boolean;
  isResizing: boolean;
  panelHeight: number;
}

export function useTerminalPanelHeight({
  isOpen,
  instantLayout,
  isResizing,
  panelHeight,
}: Options) {
  const [animatedHeight, setAnimatedHeight] = useState(0);

  useLayoutEffect(() => {
    if (instantLayout) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- le changement de session doit être atomique à l'écran
      setAnimatedHeight(isOpen ? panelHeight : 0);
      return;
    }
    let secondFrame = 0;
    if (isOpen) {
      const firstFrame = requestAnimationFrame(() => {
        secondFrame = requestAnimationFrame(() => setAnimatedHeight(panelHeight));
      });
      return () => {
        cancelAnimationFrame(firstFrame);
        if (secondFrame) cancelAnimationFrame(secondFrame);
      };
    }
    setAnimatedHeight(0);
    // panelHeight est capturé à l'ouverture ; ses changements sont gérés par l'effet suivant.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [instantLayout, isOpen]);

  const previousHeight = useRef(panelHeight);
  useEffect(() => {
    if (isOpen && !isResizing && previousHeight.current !== panelHeight) {
      setAnimatedHeight(panelHeight);
    }
    previousHeight.current = panelHeight;
  }, [panelHeight, isOpen, isResizing]);

  return { animatedHeight, setAnimatedHeight };
}
