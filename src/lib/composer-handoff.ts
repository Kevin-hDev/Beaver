/**
 * Position du champ de saisie au moment où l'on quitte l'écran d'accueil.
 *
 * L'accueil et la conversation possèdent chacun leur champ. Pour que le second
 * paraisse être le premier, il faut qu'il naisse là où l'autre se tenait — et
 * cette position, un seul endroit la détient : l'accueil l'écrit, la
 * conversation la lit une fois, elle disparaît à la lecture.
 *
 * Sans elle, l'animation devait deviner où le champ allait atterrir. Une
 * distance devinée ne peut être juste qu'à une seule taille de fenêtre : le
 * champ dépassait le bas de la page et se faisait trancher.
 */

interface HandoffPosition {
  /** Bord haut du champ, en pixels depuis le haut de la fenêtre. */
  top: number;
  at: number;
}

/* Un envoi abandonné ne doit pas faire sauter le champ d'une conversation
   ouverte trois minutes plus tard. */
const MAX_AGE_MS = 2000;

let pending: HandoffPosition | null = null;

export function noteComposerPosition(top: number): void {
  pending = { top, at: Date.now() };
}

/** Lit la position et l'efface. Rien ne subsiste d'un envoi qui n'a pas abouti. */
export function takeComposerPosition(): number | null {
  const held = pending;
  pending = null;
  return fresh(held);
}

/** Dit si une position attend, sans la consommer : la conversation a besoin de
 *  le savoir dès son premier rendu, bien avant de pouvoir mesurer quoi que ce
 *  soit. */
export function hasComposerPosition(): boolean {
  return fresh(pending) !== null;
}

function fresh(held: HandoffPosition | null): number | null {
  if (!held) return null;
  return Date.now() - held.at > MAX_AGE_MS ? null : held.top;
}
