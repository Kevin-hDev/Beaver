/**
 * La police de l'écran du terminal, et le moment où xterm peut la mesurer.
 *
 * xterm mesure la taille d'une cellule une seule fois, à l'ouverture de
 * l'écran, et ne remesure qu'au changement de `fontFamily` ou de `fontSize` :
 * il n'écoute pas le chargement des polices. Or celle de Beaver est une police
 * web. Quand l'écran s'ouvre avant qu'elle n'arrive, la cellule est mesurée sur
 * la police de secours, plus courte — puis le navigateur remplace les glyphes
 * sans que la cellule suive. Le bas des lettres qui descendent (j, g, p, q, y)
 * est alors coupé par le débordement caché des lignes, et deux largeurs de
 * caractère coexistent à l'écran.
 *
 * On ne donne donc à xterm que la police qu'il rend vraiment : celle de secours
 * tant que l'autre n'est pas chargée, la pile complète dès qu'elle l'est. Le
 * changement de famille est précisément ce qui déclenche la remesure.
 */

import { readTerminalFont } from "./terminal-theme";

/* Taille de l'écran du terminal. Elle vit ici parce que la mesure de la police
   en dépend : c'est le même choix, à un seul endroit. */
export const TERMINAL_FONT_SIZE = 13;

export interface TerminalFont {
  /** La pile à donner à xterm maintenant. */
  family: string;
  /**
   * Se résout avec la pile complète quand la police web devient utilisable.
   * Vaut `null` quand elle l'est déjà, et ne se résout jamais si elle
   * n'arrive pas : dans les deux cas la police mesurée reste celle rendue.
   */
  completed: Promise<string> | null;
}

export function resolveTerminalFont(): TerminalFont {
  const stack = readTerminalFont();
  const comma = stack.indexOf(",");
  const primary = (comma < 0 ? stack : stack.slice(0, comma)).trim();
  const fallback = comma < 0 ? "" : stack.slice(comma + 1).trim();

  /* Sans police de secours nommée, attendre n'apporterait rien : il n'y aurait
     pas d'autre famille à mesurer d'ici là. */
  if (!fallback || isLoaded(primary)) return { family: stack, completed: null };

  return { family: fallback, completed: load(primary).then(() => stack) };
}

function isLoaded(family: string): boolean {
  try {
    /* Sans API de polices — moteur ancien, environnement de test — on ne fait
       pas attendre : la pile complète est donnée tout de suite, comme avant. */
    return document.fonts?.check(shorthand(family)) ?? true;
  } catch {
    return true;
  }
}

function load(family: string): Promise<unknown> {
  try {
    return document.fonts.load(shorthand(family)).catch(never);
  } catch {
    return never();
  }
}

/* La police n'arrivera pas. La pile de secours est celle qui est rendue et elle
   a été mesurée : il n'y a rien à reprendre, donc rien à résoudre. */
function never(): Promise<never> {
  return new Promise(() => {});
}

function shorthand(family: string): string {
  return `${TERMINAL_FONT_SIZE}px ${family}`;
}
