import { useCallback, useEffect, useRef, useState } from "react";
import {
  moveId,
  sameOrder,
  slotOffsets,
  targetIndex,
  type DragSlot,
} from "@/lib/drag-reorder-geometry";

/* Glisser pour réordonner une liste — projets de la barre latérale, onglets du
   terminal. Autorité unique du geste : les positions sont photographiées une
   seule fois, à la prise, et l'ordre affiché ne change qu'au relâchement. Rien
   ne bouge donc sous la mesure pendant qu'on vise.

   L'apparence de l'élément tenu vit dans src/styles/drag-reorder.css. */

export const DRAG_ID_ATTR = "data-drag-id";
const DRAG_GROUP_ATTR = "data-drag-group";
/* Posé sur la page entière le temps d'un geste. Sa règle vit dans
   src/styles/drag-reorder.css. */
const DRAG_ACTIVE_ATTR = "data-drag-active";

/* Distance à parcourir avant qu'un appui devienne un glissement. Sans elle,
   un simple clic sur une ligne démarrait un geste complet : l'élément
   pâlissait, le curseur changeait, et l'ordre repartait s'enregistrer. */
const THRESHOLD_PX = 5;

interface DragReorderOptions {
  ids: string[];
  axis: "x" | "y";
  containerRef: React.RefObject<HTMLElement | null>;
  /* Nom de la liste. Les listes s'imbriquent — les conversations d'un projet
     vivent à l'intérieur de la liste des projets — et sans ce nom, saisir un
     projet mesurerait aussi les conversations posées dedans. */
  group: string;
  onReorder: (ids: string[], from: number, to: number) => void;
}

function coordinate(container: HTMLElement, axis: "x" | "y", clientX: number, clientY: number) {
  const box = container.getBoundingClientRect();
  return axis === "y"
    ? clientY - box.top + container.scrollTop
    : clientX - box.left + container.scrollLeft;
}

function measure(container: HTMLElement, axis: "x" | "y", group: string): DragSlot[] {
  const box = container.getBoundingClientRect();
  const base = axis === "y" ? box.top - container.scrollTop : box.left - container.scrollLeft;
  const nodes = container.querySelectorAll<HTMLElement>(`[${DRAG_ID_ATTR}]`);
  const own = Array.from(nodes).filter((el) => el.getAttribute(DRAG_GROUP_ATTR) === group);
  return own.map((el) => {
    const rect = el.getBoundingClientRect();
    return {
      id: el.getAttribute(DRAG_ID_ATTR) ?? "",
      start: (axis === "y" ? rect.top : rect.left) - base,
      size: axis === "y" ? rect.height : rect.width,
    };
  });
}

export function useDragReorder({ ids, axis, containerRef, group, onReorder }: DragReorderOptions) {
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [offsets, setOffsets] = useState<Map<string, number>>(() => new Map());
  /* Ordre obtenu au relâchement, avec celui qu'affichaient les props à ce
     moment-là. L'enregistrement fait un aller-retour par le disque : sans cet
     ordre local tenu en attendant, la liste reprendrait son ancien rang le
     temps de la réponse, puis sauterait au nouveau. Il s'efface de lui-même
     dès que les props bougent — la source de vérité reprend alors la main. */
  const [settled, setSettled] = useState<{ order: string[]; from: string[] } | null>(null);
  const order = settled && sameOrder(settled.from, ids) ? settled.order : ids;

  const grab = useRef<{ id: string; from: number; origin: number } | null>(null);
  const slots = useRef<DragSlot[]>([]);
  const target = useRef(0);
  const active = useRef(false);
  const dragged = useRef(false);
  const reorder = useRef(onReorder);
  // eslint-disable-next-line react-hooks/refs -- callback capture pattern for stable event handler
  reorder.current = onReorder;

  const stop = useCallback(() => {
    grab.current = null;
    active.current = false;
    document.body.removeAttribute(DRAG_ACTIVE_ATTR);
    setDraggingId(null);
    setOffsets(new Map());
  }, []);

  useEffect(() => {
    const onMove = (e: PointerEvent) => {
      const held = grab.current;
      const container = containerRef.current;
      if (!held || !container) return;
      const delta = coordinate(container, axis, e.clientX, e.clientY) - held.origin;
      if (!active.current) {
        if (Math.abs(delta) < THRESHOLD_PX) return;
        active.current = true;
        dragged.current = true;
        /* Le navigateur a commencé une sélection de texte pendant les premiers
           pixels : on l'efface, puis la page cesse d'en accepter le temps du
           geste. Sans quoi le déplacement surligne tout ce qu'il survole. */
        window.getSelection()?.removeAllRanges();
        document.body.setAttribute(DRAG_ACTIVE_ATTR, "true");
        setDraggingId(held.id);
      }
      target.current = targetIndex(slots.current, held.from, delta);
      setOffsets(slotOffsets(slots.current, held.from, target.current, delta));
    };

    const onUp = () => {
      const held = grab.current;
      const wasActive = active.current;
      const to = target.current;
      stop();
      if (!held || !wasActive || to === held.from) return;
      const before = slots.current.map((slot) => slot.id);
      const after = moveId(before, held.from, to);
      setSettled({ order: after, from: before });
      reorder.current(after, held.from, to);
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onUp);
      /* Démonté en plein geste, la page resterait insélectionnable. */
      document.body.removeAttribute(DRAG_ACTIVE_ATTR);
    };
  }, [axis, containerRef, stop]);

  const onPointerDown = useCallback((id: string, e: React.PointerEvent) => {
    const container = containerRef.current;
    if (e.button !== 0 || !container) return;
    dragged.current = false;
    const measured = measure(container, axis, group);
    const from = measured.findIndex((slot) => slot.id === id);
    if (from < 0) return;
    slots.current = measured;
    target.current = from;
    grab.current = { id, from, origin: coordinate(container, axis, e.clientX, e.clientY) };
  }, [axis, containerRef, group]);

  /* Posé sur la case entière : c'est elle qui se décale. */
  const itemProps = useCallback((id: string) => ({
    [DRAG_ID_ATTR]: id,
    [DRAG_GROUP_ATTR]: group,
    "data-dragging": draggingId === id ? "true" : undefined,
    style: dragStyle(offsets.get(id) ?? 0, axis, draggingId, id),
  }), [axis, draggingId, group, offsets]);

  /* Posé sur la seule zone par laquelle on peut attraper — l'en-tête d'un
     projet, et non les conversations qu'il contient. */
  const handleProps = useCallback((id: string) => ({
    onPointerDown: (e: React.PointerEvent) => onPointerDown(id, e),
  }), [onPointerDown]);

  /* Vrai jusqu'à la prise suivante : le clic arrive après le relâchement, et
     c'est là qu'un appelant doit décider de l'ignorer. */
  const didDrag = useCallback(() => dragged.current, []);

  return { order, draggingId, itemProps, handleProps, didDrag, cancel: stop };
}

export type DragItemProps = ReturnType<ReturnType<typeof useDragReorder>["itemProps"]>;
export type DragHandleProps = ReturnType<ReturnType<typeof useDragReorder>["handleProps"]>;

/* La durée est posée ici, en style calculé, et non dans une feuille partagée :
   les listes concernées déclarent déjà leur propre « transition » pour leurs
   couleurs, et une règle de même poids l'aurait remplacée — ou se serait fait
   remplacer, selon l'ordre de chargement. Elle n'existe que le temps du geste,
   et disparaît avec lui : au relâchement, la case doit rejoindre sa nouvelle
   place d'un coup, puisque la liste vient de se réordonner sous elle. */
function dragStyle(
  offset: number,
  axis: "x" | "y",
  draggingId: string | null,
  id: string,
): React.CSSProperties {
  if (!draggingId) return {};
  const transform = offset === 0
    ? undefined
    : axis === "y" ? `translateY(${offset}px)` : `translateX(${offset}px)`;
  /* Ce qu'on tient ne prend aucun retard sur le curseur ; ses voisins glissent. */
  const transition = draggingId === id ? "none" : "transform var(--ease-smooth)";
  return { transform, transition };
}
