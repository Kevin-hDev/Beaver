/**
 * Fait descendre le champ de saisie depuis la place qu'il occupait sur l'écran
 * d'accueil jusqu'à la sienne, au moment où la conversation devient visible.
 *
 * C'est le champ qui reste qui exécute le mouvement, pas celui qui part : sa
 * destination est celle où il se trouve déjà, donc rien n'est deviné. Il ne
 * peut ni dépasser, ni se faire trancher, ni atterrir à côté, quelle que soit
 * la taille de la fenêtre.
 */

import { useLayoutEffect } from "react";
import { takeComposerPosition } from "@/lib/composer-handoff";

/* Classe qui porte la durée et la courbe du glissement — elles vivent dans
   chat.css, avec le reste de l'apparence de cette colonne. */
const ARRIVING_CLASS = "chat-composer-arriving";

/* Sous le geste, la bulle du champ. La colonne se déplace d'un bloc : mesurer
   la bulle plutôt que la colonne rend le calcul insensible à ce qui s'empile
   au-dessus d'elle — panneau de tâches, demande de permission, erreur. */
const BUBBLE_SELECTOR = ".chat-input-bubble";

/** Une transition de durée nulle n'émet jamais `transitionend` : sous
 *  prefers-reduced-motion comme dans un test sans CSS, le champ doit se poser
 *  directement au lieu d'attendre une fin qui n'arrivera pas. */
function willAnimate(element: HTMLElement): boolean {
  if (typeof window === "undefined" || !window.getComputedStyle) return false;
  const durations = window.getComputedStyle(element).transitionDuration;
  if (!durations) return false;
  return durations.split(",").some((value) => Number.parseFloat(value) > 0);
}

export function useComposerHandoff(
  columnRef: React.RefObject<HTMLElement | null>,
  ready: boolean,
): void {
  useLayoutEffect(() => {
    if (!ready) return;
    const column = columnRef.current;
    if (!column) return;
    const from = takeComposerPosition();
    if (from === null) return;
    const bubble = column.querySelector<HTMLElement>(BUBBLE_SELECTOR);
    if (!bubble) return;

    const delta = from - bubble.getBoundingClientRect().top;
    if (Math.abs(delta) < 1) return;

    const settle = () => {
      column.classList.remove(ARRIVING_CLASS);
      column.style.transform = "";
    };

    /* Le champ est d'abord posé à sa position de départ, sans transition. */
    column.style.transform = `translateY(${delta}px)`;

    /* Lecture volontaire : elle oblige le navigateur à calculer cette position
       tout de suite. Sans elle, il ne verrait que l'état final et il n'y
       aurait aucun mouvement à animer. */
    column.getBoundingClientRect();

    /* La transition n'est branchée qu'ensuite, et le champ relâché dans le
       même souffle : il glisse de là où il était jusqu'à sa place. */
    column.classList.add(ARRIVING_CLASS);
    if (!willAnimate(column)) {
      settle();
      return;
    }
    column.style.transform = "";
    column.addEventListener("transitionend", settle, { once: true });

    return () => {
      column.removeEventListener("transitionend", settle);
      settle();
    };
  }, [columnRef, ready]);
}
