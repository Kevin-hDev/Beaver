import { useState, useEffect, useCallback, useRef } from "react";
import { shouldIgnoreKeyboardNavigation } from "./use-arrow-navigation";
import { isElementInsideInactiveSurface } from "@/lib/app-surface-dom";

export type NavPanel = "sidebar" | "list" | "detail" | "filePreview" | "fileTree" | "terminal";

const PANEL_ORDER: NavPanel[] = ["sidebar", "list", "detail", "filePreview", "fileTree", "terminal"];
const PANEL_FOCUS_TARGETS = [
  "[data-nav-active='true']",
  "button:not([disabled])",
  "[href]",
  "[tabindex]:not([tabindex='-1'])",
];

function panelElement(panel: NavPanel) {
  const elements = document.querySelectorAll<HTMLElement>(
    `[data-nav-zone='${panel}']`,
  );
  return Array.from(elements).find((element) => !isElementInsideInactiveSurface(element)) ?? null;
}

function panelFromTarget(target: EventTarget | null): NavPanel | null {
  if (!(target instanceof HTMLElement)) return null;
  const zone = target.closest<HTMLElement>("[data-nav-zone]")?.dataset.navZone;
  return PANEL_ORDER.includes(zone as NavPanel) ? zone as NavPanel : null;
}

function focusPanel(panel: NavPanel) {
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      const root = panelElement(panel);
      const target = PANEL_FOCUS_TARGETS
        .map((selector) => {
          if (root?.matches(selector)) return root;
          return Array.from(root?.querySelectorAll<HTMLElement>(selector) ?? [])
            .find((element) => !isElementInsideInactiveSurface(element));
        })
        .find((element): element is HTMLElement => Boolean(element));
      (target ?? root)?.focus();
    });
  });
}

function nextPanel(current: NavPanel, direction: 1 | -1): NavPanel {
  const start = PANEL_ORDER.indexOf(current);
  for (let offset = 1; offset < PANEL_ORDER.length; offset += 1) {
    const idx = start + offset * direction;
    const candidate = PANEL_ORDER[idx];
    if (candidate && panelElement(candidate)) return candidate;
  }
  return current;
}

export function usePanelFocus() {
  const [focusedPanel, setFocusedPanel] = useState<NavPanel>("list");
  const skipNextFocus = useRef(false);
  const didMount = useRef(false);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      if (shouldIgnoreKeyboardNavigation(e.target)) return;

      const sourcePanel = panelFromTarget(e.target) ?? focusedPanel;

      if (e.key === "ArrowRight") {
        e.preventDefault();
        const panel = nextPanel(sourcePanel, 1);
        setFocusedPanel(panel);
        focusPanel(panel);
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        const panel = nextPanel(sourcePanel, -1);
        setFocusedPanel(panel);
        focusPanel(panel);
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [focusedPanel]);

  useEffect(() => {
    if (!didMount.current) {
      didMount.current = true;
      return;
    }
    if (skipNextFocus.current) {
      skipNextFocus.current = false;
      return;
    }
    focusPanel(focusedPanel);
  }, [focusedPanel]);

  useEffect(() => {
    const handler = (event: FocusEvent) => {
      const panel = panelFromTarget(event.target);
      if (panel) {
        setFocusedPanel((current) => {
          if (current === panel) return current;
          skipNextFocus.current = true;
          return panel;
        });
      }
    };
    document.addEventListener("focusin", handler);
    return () => document.removeEventListener("focusin", handler);
  }, []);

  const resetToList = useCallback(() => setFocusedPanel("list"), []);

  return { focusedPanel, setFocusedPanel, resetToList };
}
