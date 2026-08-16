import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Résout les couleurs du terminal pour un thème donné, en lisant les feuilles
 * de style comme le ferait un navigateur.
 *
 * Les tests tournent dans jsdom, qui ne calcule aucune feuille de style : sans
 * cette résolution, deux couleurs écrites `color-mix(...)` se compareraient
 * comme des textes, et deux teintes identiques passeraient inaperçues.
 */

const ROOT = join(__dirname, "..", "..", "..", "..");

/** Palettes qui se posent par-dessus une base, dans l'ordre de chargement. */
const BASE: Record<string, string> = {
  "emerald-night": "dark",
  "astral-mist": "dark",
  "crimson-eclipse": "dark",
  "cobalt-frost": "light",
};

export const THEMES = [
  "dark", "light", "emerald-night", "cobalt-frost", "astral-mist", "crimson-eclipse",
];

export interface Rgb { r: number; g: number; b: number }

function readTokens(file: string): Map<string, string> {
  /* eslint-disable-next-line security/detect-non-literal-fs-filename -- le nom
     vient de THEMES, liste fermée de fichiers du dépôt. */
  const css = readFileSync(join(ROOT, "src/styles/themes", `${file}.css`), "utf8");
  const tokens = new Map<string, string>();
  for (const [, name, value] of css.matchAll(/(--[\w-]+):\s*([^;]+);/g)) {
    tokens.set(name, value.trim());
  }
  return tokens;
}

function paletteTokens(): Map<string, string> {
  /* eslint-disable-next-line security/detect-non-literal-fs-filename -- chemin
     fixe, seule la racine du dépôt est calculée. */
  const css = readFileSync(join(ROOT, "src/components/terminal/terminal-palette.css"), "utf8");
  const tokens = new Map<string, string>();
  for (const [, name, value] of css.matchAll(/(--term-[\w-]+):\s*([^;]+);/g)) {
    tokens.set(name, value.trim());
  }
  return tokens;
}

function tokensFor(theme: string): Map<string, string> {
  const merged = new Map<string, string>();
  const base = BASE[theme];
  const layers = base ? [base, theme] : [theme];
  for (const layer of layers) {
    for (const [name, value] of readTokens(layer)) merged.set(name, value);
  }
  for (const [name, value] of paletteTokens()) merged.set(name, value);
  return merged;
}

function parseHex(value: string): Rgb | null {
  const match = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(value);
  if (!match) return null;
  const hex = match[1].length === 3 ? [...match[1]].map((c) => c + c).join("") : match[1];
  return {
    r: parseInt(hex.slice(0, 2), 16),
    g: parseInt(hex.slice(2, 4), 16),
    b: parseInt(hex.slice(4, 6), 16),
  };
}

/* Une couleur translucide est aplatie sur le fond qu'elle recouvre : c'est ce
   que l'œil voit, et donc ce qu'il faut comparer. */
function parseRgba(value: string, over: Rgb | null): Rgb | null {
  const match = /^rgba?\(([^)]+)\)$/.exec(value);
  if (!match) return null;
  const parts = match[1].split(",").map((p) => Number.parseFloat(p.trim()));
  const [r, g, b, a = 1] = parts;
  if (a >= 1 || !over) return { r, g, b };
  return {
    r: r * a + over.r * (1 - a),
    g: g * a + over.g * (1 - a),
    b: b * a + over.b * (1 - a),
  };
}

const MIX_PREFIX = "color-mix(in srgb,";

function mix(value: string, tokens: Map<string, string>, over: Rgb | null): Rgb | null {
  if (!value.startsWith(MIX_PREFIX) || !value.endsWith(")")) return null;
  const [first, second] = splitTop(value.slice(MIX_PREFIX.length, -1));
  const share = (part: string) => {
    const pct = /([\d.]+)%$/.exec(part.trim());
    return pct ? Number.parseFloat(pct[1]) / 100 : null;
  };
  const strip = (part: string) => part.trim().replace(/[\d.]+%$/, "").trim();
  const firstShare = share(first) ?? (share(second) !== null ? 1 - share(second)! : 0.5);
  const a = resolve(strip(first), tokens, over);
  const b = strip(second) === "transparent" ? over : resolve(strip(second), tokens, over);
  if (!a || !b) return null;
  return {
    r: a.r * firstShare + b.r * (1 - firstShare),
    g: a.g * firstShare + b.g * (1 - firstShare),
    b: a.b * firstShare + b.b * (1 - firstShare),
  };
}

/** Découpe sur la virgule de premier niveau, en ignorant celles des `var()`. */
function splitTop(value: string): [string, string] {
  let depth = 0;
  for (let i = 0; i < value.length; i += 1) {
    if (value[i] === "(") depth += 1;
    else if (value[i] === ")") depth -= 1;
    else if (value[i] === "," && depth === 0) {
      return [value.slice(0, i).trim(), value.slice(i + 1).trim()];
    }
  }
  return [value.trim(), ""];
}

export function resolve(
  value: string,
  tokens: Map<string, string>,
  over: Rgb | null = null,
  depth = 0,
): Rgb | null {
  if (depth > 12) return null;
  const trimmed = value.trim();
  if (trimmed.startsWith("var(") && trimmed.endsWith(")")) {
    const [name, fallback] = splitTop(trimmed.slice(4, -1));
    const referenced = tokens.get(name.trim()) ?? fallback;
    return referenced ? resolve(referenced, tokens, over, depth + 1) : null;
  }
  return parseHex(trimmed) ?? parseRgba(trimmed, over) ?? mix(trimmed, tokens, over);
}

/** Les seize couleurs de sortie d'un thème, aplaties sur son fond. */
export function ansiColors(theme: string): Map<string, Rgb> {
  const tokens = tokensFor(theme);
  const background = resolve("var(--void)", tokens);
  const names = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"];
  const all = [...names, ...names.map((n) => `bright-${n}`)];
  const colors = new Map<string, Rgb>();
  for (const name of all) {
    const color = resolve(`var(--term-${name})`, tokens, background);
    if (color) colors.set(name, color);
  }
  return colors;
}

export function backgroundOf(theme: string): Rgb | null {
  return resolve("var(--void)", tokensFor(theme));
}
