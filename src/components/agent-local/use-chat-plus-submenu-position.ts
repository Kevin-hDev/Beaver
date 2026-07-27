import { useLayoutEffect, useState, type RefObject } from "react";

export function useChatPlusSubmenuPosition(
  open: boolean,
  submenu: string | null,
  wrapperRef: RefObject<HTMLDivElement | null>,
  dropdownRef: RefObject<HTMLDivElement | null>,
  submenuRef: RefObject<HTMLDivElement | null>,
) {
  const [left, setLeft] = useState(0);

  useLayoutEffect(() => {
    if (!open || !submenu) return;
    const position = () => {
      const dropdown = dropdownRef.current;
      const submenuElement = submenuRef.current;
      const wrapper = wrapperRef.current;
      if (!dropdown || !submenuElement || !wrapper) return;
      const rootStyle = getComputedStyle(document.documentElement);
      const gap = Number.parseFloat(rootStyle.getPropertyValue("--space-xs")) || 0;
      const dropdownRect = dropdown.getBoundingClientRect();
      const wrapperRect = wrapper.getBoundingClientRect();
      const right = dropdown.offsetWidth + gap;
      const leftSide = -submenuElement.offsetWidth - gap;
      const fitsRight = dropdownRect.right + gap + submenuElement.offsetWidth <= window.innerWidth;
      setLeft(Math.max(fitsRight ? right : leftSide, -wrapperRect.left + gap));
    };
    position();
    window.addEventListener("resize", position);
    return () => window.removeEventListener("resize", position);
  }, [dropdownRef, open, submenu, submenuRef, wrapperRef]);

  return left;
}
