export function isElementInsideInactiveSurface(element: Element): boolean {
  return element.closest("[hidden], [inert], [aria-hidden='true']") !== null;
}
