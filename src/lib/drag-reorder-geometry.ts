/* Géométrie du réordonnancement par glissement.

   Fonctions pures : elles ne lisent jamais l'écran. Elles reçoivent la photo
   des positions prise au moment où l'on attrape, et ne travaillent plus que
   sur elle. C'est la raison d'être du fichier — mesurer une cible pendant que
   le glissement la déplace est précisément ce qui rendait les deux listes
   imprécises : l'élément visé s'écartait, on croyait l'avoir quitté, on le
   remettait en place, il repassait sous le curseur. */

export interface DragSlot {
  id: string;
  /** Bord haut (ou gauche) de la case, dans le repère du conteneur. */
  start: number;
  /** Hauteur (ou largeur) de la case. */
  size: number;
}

/** Espace entre deux cases voisines. Le premier trouvé fait foi : ces listes
    posent le même écart partout, et une liste d'un seul élément n'en a aucun. */
export function slotGap(slots: DragSlot[]): number {
  for (let i = 1; i < slots.length; i++) {
    const gap = slots[i].start - (slots[i - 1].start + slots[i - 1].size);
    if (gap > 0) return gap;
  }
  return 0;
}

/* Le déplacement ne sort pas de la liste. Deux raisons : au-delà du dernier
   voisin il ne se passe plus rien, et le conteneur des projets rogne ce qui
   dépasse — un projet emmené trop loin s'y couperait en deux. */
export function clampDelta(slots: DragSlot[], from: number, delta: number): number {
  const dragged = slots[from];
  const last = slots[slots.length - 1];
  if (!dragged || !last) return delta;
  const lowest = slots[0].start - dragged.start;
  const highest = last.start + last.size - (dragged.start + dragged.size);
  return Math.min(Math.max(delta, lowest), highest);
}

/* La case visée est celle dont on a franchi le milieu, pas celle qu'on
   effleure : un projet déplié occupe dix fois la surface d'un projet replié,
   et déclencher à l'entrée rendait le geste imprévisible selon le voisin. */
export function targetIndex(slots: DragSlot[], from: number, delta: number): number {
  const dragged = slots[from];
  if (!dragged) return from;
  const center = dragged.start + dragged.size / 2 + delta;
  let target = from;
  for (let i = 0; i < slots.length; i++) {
    if (i === from) continue;
    const other = slots[i].start + slots[i].size / 2;
    if (i < from && center < other) target = Math.min(target, i);
    if (i > from && center > other) target = Math.max(target, i);
  }
  return target;
}

export function moveId(ids: string[], from: number, to: number): string[] {
  const next = [...ids];
  const [moved] = next.splice(from, 1);
  next.splice(to, 0, moved);
  return next;
}

/* Décalage à appliquer à chaque case pendant le geste. Ce qu'on tient suit le
   curseur ; ses voisins libèrent ou comblent la place laissée, qui vaut
   toujours la taille de la case déplacée, quelle que soit la leur. */
export function slotOffsets(
  slots: DragSlot[],
  from: number,
  to: number,
  delta: number,
): Map<string, number> {
  const offsets = new Map<string, number>();
  const dragged = slots[from];
  if (!dragged) return offsets;
  offsets.set(dragged.id, delta);
  const shift = dragged.size + slotGap(slots);
  for (let i = 0; i < slots.length; i++) {
    if (i === from) continue;
    if (i > from && i <= to) offsets.set(slots[i].id, -shift);
    if (i < from && i >= to) offsets.set(slots[i].id, shift);
  }
  return offsets;
}

export function sameOrder(a: string[], b: string[]): boolean {
  return a.length === b.length && a.every((id, i) => id === b[i]);
}
