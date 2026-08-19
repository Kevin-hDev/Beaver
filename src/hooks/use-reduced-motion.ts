import { useSyncExternalStore } from "react";

const QUERY = "(prefers-reduced-motion: reduce)";

/* Le réglage système « réduire les animations », lu depuis React.

   global.css coupe déjà toute animation CSS quand il est actif. Les animations
   SMIL portées par un SVG lui échappent : aucune règle CSS ne les atteint, et
   ne pas les rendre du tout est le seul moyen de les arrêter. D'où ce détour
   par JavaScript pour un réglage qui se traite ailleurs en feuille de style. */
export function usePrefersReducedMotion(): boolean {
  return useSyncExternalStore(subscribe, readSetting, () => false);
}

function subscribe(onChange: () => void): () => void {
  const media = window.matchMedia?.(QUERY);
  if (!media) return () => {};
  media.addEventListener("change", onChange);
  return () => media.removeEventListener("change", onChange);
}

function readSetting(): boolean {
  return window.matchMedia?.(QUERY).matches ?? false;
}
