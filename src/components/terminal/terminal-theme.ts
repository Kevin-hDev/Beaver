/**
 * Apparence de l'écran du terminal — couleurs et police — lue depuis les jetons
 * de l'application.
 *
 * xterm peint son écran hors des feuilles de style : rien de ce qu'il affiche
 * ne peut être décidé en CSS. Ce fichier est le passage obligé entre les deux
 * mondes, et le seul endroit du terminal qui nomme une couleur.
 */

import type { ITheme } from "@xterm/xterm";
import "./terminal-palette.css";

/* Les seize couleurs ANSI, dans l'ordre où xterm les nomme. */
const ANSI = [
  "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
  "brightBlack", "brightRed", "brightGreen", "brightYellow",
  "brightBlue", "brightMagenta", "brightCyan", "brightWhite",
] as const;

/* Jeton correspondant, dans le même ordre. */
const ANSI_TOKENS = ANSI.map(
  (name) => `--term-${name.replace(/([A-Z])/g, "-$1").toLowerCase()}`,
);

/**
 * Résout des jetons de couleur en valeurs concrètes.
 *
 * `getPropertyValue` rend le texte écrit dans la feuille de style, pas la
 * couleur : un `color-mix` en ressort tel quel, et xterm ne sait pas le lire.
 * Une sonde invisible qui porte la couleur, elle, la rend calculée.
 */
function resolveColors(tokens: string[]): string[] {
  const probe = document.createElement("span");
  probe.style.display = "none";
  document.body.appendChild(probe);
  try {
    return tokens.map((token) => {
      probe.style.color = "";
      probe.style.color = `var(${token})`;
      return getComputedStyle(probe).color;
    });
  } finally {
    probe.remove();
  }
}

function tokenText(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

export function readTerminalTheme(): ITheme {
  const [background, foreground, cursor, selection] = resolveColors([
    "--void", "--ink", "--term-cursor", "--term-selection",
  ]);
  const ansi = resolveColors(ANSI_TOKENS);
  const theme: ITheme = {
    background,
    foreground,
    cursor,
    cursorAccent: background,
    selectionBackground: selection,
  };
  ANSI.forEach((name, index) => {
    theme[name] = ansi[index];
  });
  return theme;
}

/**
 * La police du terminal est celle de l'application, lue à l'exécution.
 *
 * Elle était nommée en dur, et sous un nom que la police installée ne porte
 * pas : la famille n'était jamais trouvée et le terminal écrivait dans la
 * police par défaut du système, seul écran de l'application à le faire.
 */
export function readTerminalFont(): string {
  return tokenText("--font-mono", "ui-monospace, monospace");
}
