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
      return toXtermColor(getComputedStyle(probe).color);
    });
  } finally {
    probe.remove();
  }
}

/**
 * Remet une couleur calculée dans la seule écriture que xterm sait lire.
 *
 * Il n'analyse lui-même que `#rrggbb[aa]` et `rgb()/rgba()` séparés par des
 * virgules ; tout le reste passe par un essai sur un canevas, qui rejette les
 * couleurs translucides. Or `color-mix` se calcule en `color(srgb …)` dans les
 * deux moteurs : la couleur de sélection de Beaver, translucide, était refusée
 * en silence et remplacée par le blanc d'usine de xterm.
 *
 * Seules les écritures à espaces sont retouchées : celle à virgules est déjà
 * lisible par xterm, et tout ce qui ne se lit pas ici ressort tel quel, donc
 * comme avant.
 */
export function toXtermColor(css: string): string {
  const written = /^(color\(srgb|rgba?\()([^)]*)\)$/.exec(css);
  if (!written) return css;

  const [values, alpha] = written[2].split("/");
  const channels = values.trim().split(/ +/).map(Number);
  /* Les canaux de `color(srgb …)` vont de 0 à 1, ceux de `rgb()` de 0 à 255. */
  const scale = written[1].startsWith("color") ? 255 : 1;
  if (channels.length !== 3 || !channels.every(Number.isFinite)) return css;

  /* CSS accepte les canaux hors gamut ; xterm les encode sans les borner. */
  const [red, green, blue] = channels.map((value) => Math.round(Math.min(255, Math.max(0, value * scale))));
  const parsedAlpha = readAlpha(alpha);
  if (!Number.isFinite(parsedAlpha)) return css;
  const opacity = Math.min(1, Math.max(0, parsedAlpha));

  return opacity >= 1
    ? `rgb(${red}, ${green}, ${blue})`
    : `rgba(${red}, ${green}, ${blue}, ${opacity})`;
}

function readAlpha(alpha: string | undefined): number {
  if (alpha === undefined) return 1;
  const trimmed = alpha.trim();
  return trimmed.endsWith("%") ? Number(trimmed.slice(0, -1)) / 100 : Number(trimmed);
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
