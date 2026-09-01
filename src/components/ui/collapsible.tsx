import { useLayoutEffect, useRef, useState } from "react";
import type { ReactNode, TransitionEvent } from "react";
import { cn } from "@/lib/utils";
import "./collapsible.css";

interface CollapsibleProps {
  open: boolean;
  /** Retire le contenu du DOM une fois le repli terminé. */
  unmountWhenClosed?: boolean;
  className?: string;
  innerClassName?: string;
  children: ReactNode;
}

/** Une transition ne se déclenche pas si sa durée est nulle : ni sous
 *  prefers-reduced-motion, ni dans un environnement de test sans CSS. Sans ce
 *  test, `transitionend` n'arriverait jamais et l'état resterait figé. */
function willAnimate(element: HTMLElement): boolean {
  if (typeof window === "undefined" || !window.getComputedStyle) return false;
  const durations = window.getComputedStyle(element).transitionDuration;
  if (!durations) return false;
  return durations.split(",").some((value) => Number.parseFloat(value) > 0);
}

/** État de repos, hors animation. Ouvert, la hauteur redevient automatique pour
 *  suivre un contenu qui grandit ensuite, et l'overflow est relâché pour ne pas
 *  trancher les couches flottantes (infobulles, menus). */
function settle(region: HTMLElement, open: boolean): void {
  region.style.height = open ? "auto" : "0px";
  region.style.overflow = open ? "visible" : "hidden";
}

/**
 * Zone repliable animée, primitive unique de l'application.
 *
 * L'animation porte sur `height` en pixels réels et non sur une piste de grille
 * en `fr` : WebKit — le moteur de rendu de l'app sur macOS — interpole les
 * unités `fr` par paliers, ce qui hachait le début du dépliement et la fin du
 * repliement, là où la courbe est la plus lente.
 */
export function Collapsible({
  open,
  unmountWhenClosed = false,
  className,
  innerClassName,
  children,
}: CollapsibleProps) {
  const regionRef = useRef<HTMLDivElement>(null);
  const innerRef = useRef<HTMLDivElement>(null);
  const [mounted, setMounted] = useState(open);
  const isFirstRender = useRef(true);

  // Le contenu doit exister dans le DOM avant que sa hauteur soit mesurable :
  // on le monte pendant le rendu qui ouvre, pas dans un effet qui suivrait.
  if (open && !mounted) setMounted(true);

  const finish = () => {
    const region = regionRef.current;
    if (!region) return;
    settle(region, open);
    if (!open && unmountWhenClosed) setMounted(false);
  };

  useLayoutEffect(() => {
    const region = regionRef.current;
    if (!region) return;
    if (isFirstRender.current) {
      isFirstRender.current = false;
      settle(region, open);
      return;
    }

    // Repart de la hauteur affichée à l'instant présent, pas de la hauteur
    // théorique : une animation interrompue en plein vol enchaîne sans saut.
    const from = region.getBoundingClientRect().height;
    const to = open ? (innerRef.current?.getBoundingClientRect().height ?? 0) : 0;

    region.style.overflow = "hidden";
    region.style.height = `${from}px`;
    // Lecture forcée : sans elle le navigateur fusionne les deux écritures de
    // hauteur et passe directement à la cible, sans rien animer.
    void region.offsetHeight;
    region.style.height = `${to}px`;

    if (!willAnimate(region)) finish();
    // `finish` est recréé à chaque rendu et lit donc déjà le `open` courant ;
    // le déclarer en dépendance relancerait l'animation à chaque rendu du parent.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const handleTransitionEnd = (event: TransitionEvent<HTMLElement>) => {
    if (event.target !== event.currentTarget) return;
    if (event.propertyName !== "height") return;
    finish();
  };

  return (
    <div
      ref={regionRef}
      className={cn("cps-region", className)}
      /* L'état de repos est annoncé dès le rendu, pas dans un effet : les
       * effets des enfants s'exécutent avant celui du parent, et un contenu
       * qui se mesure lui-même (un graphe, par exemple) se dessinerait dans
       * une région encore haute de zéro. Les styles écrits pendant
       * l'animation restent prioritaires, ils sont en ligne. */
      data-open={open ? "true" : "false"}
      onTransitionEnd={handleTransitionEnd}
    >
      {(mounted || !unmountWhenClosed) && (
        <div ref={innerRef} className={cn("cps-inner", innerClassName)}>
          {children}
        </div>
      )}
    </div>
  );
}
