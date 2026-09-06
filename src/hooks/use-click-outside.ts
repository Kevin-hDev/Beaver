import { useEffect, type RefObject } from "react";

/* floatingRef : la couche portée ailleurs dans le document par un portail. Sans
   elle, un panneau sorti de son conteneur compte comme « dehors » et le premier
   clic sur une option le referme avant que la sélection ne parte. */
export function useClickOutside(
  ref: RefObject<HTMLElement | null>,
  onClickOutside: () => void,
  floatingRef?: RefObject<HTMLElement | null>,
) {
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (!ref.current) return;
      const target = e.target as Node;
      if (ref.current.contains(target)) return;
      if (floatingRef?.current?.contains(target)) return;
      onClickOutside();
    }

    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [floatingRef, onClickOutside, ref]);
}
