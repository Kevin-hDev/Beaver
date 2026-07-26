interface TransitionGuards {
  moving: () => boolean;
  detach: () => void;
}

const LAYOUT_TARGET_SELECTOR = [
  ".app-root",
  ".app-sidebar-block",
  '[data-nav-zone="sidebar"]',
  ".app-list-panel",
  ".app-detail-panel",
  ".agent-detail-with-preview",
  ".agent-detail-chat",
  ".asp-panel",
  ".asp-slide-wrapper",
  ".ft-panel",
].join(",");

const TRACKED_PROPERTIES = new Set([
  "flex-grow",
  "margin-left",
  "max-width",
  "min-width",
  "padding-left",
  "transform",
  "width",
]);

function transitionTarget(event: TransitionEvent): HTMLElement | null {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return null;
  if (!target.matches(LAYOUT_TARGET_SELECTOR)) return null;
  return TRACKED_PROPERTIES.has(event.propertyName) ? target : null;
}

export function attachBrowserTransitionGuards(
  host: HTMLDivElement,
  onBegin: () => void,
  onEnd: () => void,
): TransitionGuards {
  const panel = host.closest<HTMLElement>(".asp-panel");
  const root = host.closest<HTMLElement>(".app-root")
    ?? host.closest<HTMLElement>(".agent-detail-with-preview")
    ?? panel;
  if (!root) return { moving: () => false, detach: () => {} };

  const active = new Map<HTMLElement, Set<string>>();
  const begin = (event: TransitionEvent) => {
    const target = transitionTarget(event);
    if (!target) return;
    const wasMoving = active.size > 0;
    const properties = active.get(target) ?? new Set<string>();
    properties.add(event.propertyName);
    active.set(target, properties);
    if (!wasMoving) onBegin();
  };
  const end = (event: TransitionEvent) => {
    const target = transitionTarget(event);
    if (!target) return;
    const properties = active.get(target);
    if (!properties?.delete(event.propertyName)) return;
    if (properties.size === 0) active.delete(target);
    if (active.size === 0) onEnd();
  };

  root.addEventListener("transitionrun", begin);
  root.addEventListener("transitionend", end);
  root.addEventListener("transitioncancel", end);

  const mutationObserver = typeof MutationObserver === "undefined"
    ? null
    : new MutationObserver(() => {
      if (active.size === 0) onEnd();
    });
  const mutationTargets = [
    ...(root.matches(LAYOUT_TARGET_SELECTOR) ? [root] : []),
    ...root.querySelectorAll<HTMLElement>(LAYOUT_TARGET_SELECTOR),
  ];
  for (const target of mutationTargets) {
    mutationObserver?.observe(target, { attributes: true, attributeFilter: ["class", "style"] });
  }

  return {
    moving: () => active.size > 0,
    detach: () => {
      mutationObserver?.disconnect();
      root.removeEventListener("transitionrun", begin);
      root.removeEventListener("transitionend", end);
      root.removeEventListener("transitioncancel", end);
    },
  };
}
