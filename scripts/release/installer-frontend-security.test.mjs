import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readdir, readFile } from "node:fs/promises";
import test from "node:test";

const SOURCE = new URL("../../installer/src/", import.meta.url);
const SVG = new URL("../../src/assets/QPXAq01-anime.svg", import.meta.url);

async function installerSources() {
  const names = (await readdir(SOURCE)).filter((name) => /\.(?:ts|tsx|css)$/u.test(name));
  return Promise.all(names.map(async (name) => [name, await readFile(new URL(name, SOURCE), "utf8")]));
}

test("le castor embarqué reste l'asset canonique inerte", async () => {
  const svg = await readFile(SVG, "utf8");
  assert.equal(
    createHash("sha256").update(svg).digest("hex"),
    "13d902f486f663be22c1f7a1ca8723a8d763b924884b3cb36720e20c17bc91f8",
  );
  assert.doesNotMatch(svg, /<(?:script|foreignObject)\b/iu);
  assert.doesNotMatch(svg, /\son[a-z]+\s*=/iu);
  assert.doesNotMatch(svg, /(?:href|src)\s*=\s*["'](?:https?:|\/\/)/iu);
  assert.match(svg, /prefers-reduced-motion/u);
});

test("l'injection SVG est unique, locale et sans navigation dynamique", async () => {
  const sources = await installerSources();
  const combined = sources.map(([name, source]) => `/* ${name} */\n${source}`).join("\n");
  assert.equal(combined.match(/dangerouslySetInnerHTML/gu)?.length, 1);
  const beaver = await readFile(new URL("installer-beaver.tsx", SOURCE), "utf8");
  assert.match(beaver, /QPXAq01-anime\.svg\?raw/u);
  assert.match(beaver, /__html: beaverSvg/u);
  assert.doesNotMatch(
    combined.replace("dangerouslySetInnerHTML", ""),
    /(?:\.innerHTML\s*=|\beval\s*\(|new\s+Function\b|<script[^>]+https?:|location\.(?:href|assign|replace)\s*[=(]|window\.open\s*\()/iu,
  );
  assert.doesNotMatch(combined, /banc-chassis|banc-install|beaver-styles\.css/u);
});

test("le serveur de développement autorise les traductions partagées", async () => {
  const viteConfig = await readFile(new URL("../../installer/vite.config.ts", import.meta.url), "utf8");
  assert.match(viteConfig, /path\.resolve\(installerRoot, "\.\.\/src\/i18n"\)/u);
});
