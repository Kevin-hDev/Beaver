/**
 * Sortie de l'écran d'accueil : on attend que le titre ait fini de se dissoudre
 * avant de laisser la place à la conversation.
 *
 * La durée de cette dissolution vit dans welcome-wordmark.css et n'est recopiée
 * nulle part ici : c'est la fin réelle de l'animation qu'on attend. Le délai
 * n'est qu'une borne — sous mouvement réduit l'animation est désactivée, et un
 * envoi ne doit jamais rester suspendu à une fin qui n'arrivera pas.
 */

const LEAVE_BOUND_MS = 600;

const TITLE_SELECTOR = ".wm-title";

/** Une animation de durée nulle n'émet jamais `animationend`. */
function willAnimate(element: HTMLElement): boolean {
  if (typeof window === "undefined" || !window.getComputedStyle) return false;
  const durations = window.getComputedStyle(element).animationDuration;
  if (!durations) return false;
  return durations.split(",").some((value) => Number.parseFloat(value) > 0);
}

export function waitForTitleExit(content: HTMLElement | null): Promise<void> {
  const title = content?.querySelector<HTMLElement>(TITLE_SELECTOR);
  if (!title || !willAnimate(title)) return Promise.resolve();
  return new Promise((resolve) => {
    const done = () => {
      clearTimeout(bound);
      title.removeEventListener("animationend", done);
      resolve();
    };
    const bound = setTimeout(done, LEAVE_BOUND_MS);
    title.addEventListener("animationend", done);
  });
}
