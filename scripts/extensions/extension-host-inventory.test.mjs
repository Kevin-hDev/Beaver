import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { relative, resolve } from "node:path";
import test from "node:test";

const hostRoot = resolve("src-tauri/resources/extension-host");
const imageSizeDirectory = "vendor/image-size-disabled";
const builtinCatalog = JSON.parse(
  readFileSync(resolve(hostRoot, "builtin-plugins/catalog.json"), "utf8"),
);
const imageSizePackage = JSON.parse(
  readFileSync(resolve(hostRoot, imageSizeDirectory, "package.json"), "utf8"),
);
const entrypoints = [
  "host.mjs",
  ...builtinCatalog.plugins.map((plugin) => plugin.manifest.main),
  `${imageSizeDirectory}/${imageSizePackage.exports.import.slice(2)}`,
];
// Cette dépendance remplace le paquet npm image-size attendu par pptxgenjs.
const allowedRuntimeArtifacts = new Set([
  "sdk/index.mjs",
  // Rust launches the copied UI builder directly; it is not part of the Host
  // module graph rooted at host.mjs.
  "ui-build.mjs",
  `${imageSizeDirectory}/${imageSizePackage.main.slice(2)}`,
]);
// npm and the bundled Node runtime are pinned and exercised by their dedicated
// preparation/smoke tests. This inventory owns Beaver-authored Host sources.
const externalRuntimePrefixes = ["node_modules/", "runtime/"];

test("every bundled host JavaScript file has a bounded production path", () => {
  const reached = new Set(entrypoints);
  const pending = [...entrypoints];
  while (pending.length > 0) {
    const current = pending.pop();
    const source = readFileSync(resolve(hostRoot, current), "utf8");
    for (const target of relativeImports(source)) {
      const next = portablePath(
        relative(hostRoot, resolve(hostRoot, current, "..", target)),
      );
      if (!reached.has(next)) {
        reached.add(next);
        pending.push(next);
      }
    }
  }

  const bundled = readdirSync(hostRoot, { recursive: true })
    .map(portablePath)
    .filter((entry) => /\.(?:mjs|js|cjs)$/u.test(entry))
    .filter((entry) => !externalRuntimePrefixes.some((prefix) => entry.startsWith(prefix)))
    .sort();
  const accountedFor = new Set([...reached, ...allowedRuntimeArtifacts]);
  assert.deepEqual(bundled.filter((entry) => !accountedFor.has(entry)), []);
});

function relativeImports(source) {
  return [
    ...source.matchAll(/(?:from\s*|import\s*\()(["'])(\.[^"']+\.(?:mjs|js|cjs))\1/g),
    ...source.matchAll(/new URL\((["'])(\.[^"']+\.(?:mjs|js|cjs))\1/g),
  ].map((match) => match[2]);
}

function portablePath(path) {
  return path.replaceAll("\\", "/");
}
