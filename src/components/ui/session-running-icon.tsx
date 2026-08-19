import { usePrefersReducedMotion } from "@/hooks/use-reduced-motion";
import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Repère d'une session qui répond : vingt-cinq carrés en grille, chacun
   immobile à sa place. Une aiguille part de midi et fait le tour dans le sens
   horaire ; chaque carré se rétracte vers son centre à son passage, puis se
   rallume. Les carrés d'un même rayon partent ensemble — c'est ce qui donne une
   aiguille qui tourne plutôt qu'un scintillement. Le tour fini, la grille reste
   pleine un instant avant de repartir.

   Le retard de chaque carré est calculé à partir de son angle et non écrit à la
   main : la géométrie reste juste si la grille change de taille, et les vingt-
   cinq retards ne peuvent pas diverger d'un réglage à l'autre.

   Chaque animation boucle sur elle-même, sans référence à une autre : plusieurs
   sessions tournent en même temps dans la barre latérale, et des animations qui
   se synchroniseraient par identifiant en partageraient un entre toutes. */

const GRID = 5;
const VIEWBOX = 24;
const MARGIN = 1;
const CELL = (VIEWBOX - 2 * MARGIN) / GRID;
/* Ce qu'il reste d'un carré au creux de sa rétraction, en proportion. */
const SHRINK = 0.18;
/* Arrondi des quatre coins extérieurs de la grille, en proportion d'un carré.
   Porté par les quatre carrés d'angle seulement, et sur leur coin extérieur
   seulement : arrondir les coins intérieurs creuserait des encoches là où deux
   carrés se touchent. L'arrondi se rétracte avec son carré, puisqu'il est
   dessiné dedans et non découpé par-dessus. */
const CORNER = 0.4;
/* Durée d'un tour d'aiguille, de la rétraction d'un carré, et du repos pendant
   lequel la grille reste pleine avant le tour suivant. */
const SWEEP_S = 1.5;
const PULSE_S = 0.45;
const REST_S = 0.3;

interface Cell {
  cx: number;
  cy: number;
  delay: number;
  /* Rayon de chaque coin, dans le sens horaire depuis le coin haut-gauche. */
  corners: [number, number, number, number];
}

const CELLS: readonly Cell[] = buildCells();
const CYCLE_S = Math.max(...CELLS.map((cell) => cell.delay)) + PULSE_S + REST_S;

function buildCells(): Cell[] {
  const cells: Cell[] = [];
  for (let row = 0; row < GRID; row += 1) {
    for (let col = 0; col < GRID; col += 1) {
      const cx = MARGIN + CELL * (col + 0.5);
      const cy = MARGIN + CELL * (row + 0.5);
      const dx = cx - VIEWBOX / 2;
      const dy = cy - VIEWBOX / 2;
      /* Angle mesuré depuis midi et croissant vers la droite, donc horaire. La
         case du centre n'en a pas : elle bat avec le rayon de midi. */
      const angle = dx === 0 && dy === 0
        ? 0
        : (Math.atan2(dx, -dy) + 2 * Math.PI) % (2 * Math.PI);
      const last = GRID - 1;
      cells.push({
        cx,
        cy,
        delay: (angle / (2 * Math.PI)) * SWEEP_S,
        corners: [
          row === 0 && col === 0 ? CELL * CORNER : 0,
          row === 0 && col === last ? CELL * CORNER : 0,
          row === last && col === last ? CELL * CORNER : 0,
          row === last && col === 0 ? CELL * CORNER : 0,
        ],
      });
    }
  }
  return cells;
}

/* La rétraction est décrite en repères sur le cycle entier plutôt qu'en date de
   départ, pour la même raison : une date de départ décalée demanderait de
   pointer une autre animation, donc un identifiant partagé. */
function pulse(delay: number): { keyTimes: string; values: string } {
  const marks = [0, delay, delay + PULSE_S / 2, delay + PULSE_S, CYCLE_S];
  const scales = [1, 1, SHRINK, 1, 1];
  const keyTimes: number[] = [];
  const values: number[] = [];
  marks.forEach((mark, i) => {
    /* Deux repères au même instant décriraient un saut. Le premier fait double
       emploi avec le suivant quand la rétraction démarre au début du cycle. */
    if (i > 0 && mark === marks[i - 1]) {
      keyTimes.pop();
      values.pop();
    }
    keyTimes.push(mark / CYCLE_S);
    values.push(scales[i]);
  });
  return { keyTimes: keyTimes.map(round).join(";"), values: values.map(round).join(";") };
}

/* Un carré dessiné autour de son centre, chaque coin avec son propre rayon. Un
   rayon nul donne un arc de rayon nul, que SVG rend comme un trait droit : les
   vingt et un carrés sans arrondi passent par le même tracé que les quatre
   autres, et il n'y a qu'une géométrie de carré dans ce fichier. */
function square([tl, tr, br, bl]: [number, number, number, number]): string {
  const h = CELL / 2;
  const arc = (r: number, x: number, y: number) => `a${round(r)} ${round(r)} 0 0 1 ${round(x)} ${round(y)}`;
  return [
    `M${round(-h + tl)} ${round(-h)}`,
    `H${round(h - tr)}`, arc(tr, tr, tr),
    `V${round(h - br)}`, arc(br, -br, br),
    `H${round(-h + bl)}`, arc(bl, -bl, -bl),
    `V${round(-h + tl)}`, arc(tl, tl, -tl),
    "Z",
  ].join(" ");
}

function round(value: number): string {
  return String(Number(value.toFixed(4)));
}

export function SessionRunningIcon({
  size = "var(--session-running-icon-size)",
  className,
}: InlineIconProps) {
  const still = usePrefersReducedMotion();
  return (
    <InlineIcon size={size} className={className} viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}>
      {CELLS.map((cell) => {
        const { keyTimes, values } = pulse(cell.delay);
        return (
          /* Le carré est dessiné autour de son propre centre et déplacé par son
             groupe : une mise à l'échelle prend l'origine du repère, et sans ce
             découpage elle tirerait chaque carré vers le coin du cadre. */
          <g key={`${cell.cx}-${cell.cy}`} transform={`translate(${round(cell.cx)} ${round(cell.cy)})`}>
            <path d={square(cell.corners)} fill="currentColor">
              {!still && (
                <animateTransform
                  attributeName="transform"
                  type="scale"
                  dur={`${round(CYCLE_S)}s`}
                  repeatCount="indefinite"
                  keyTimes={keyTimes}
                  values={values}
                />
              )}
            </path>
          </g>
        );
      })}
    </InlineIcon>
  );
}
