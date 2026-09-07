import { describe, expect, it } from "vitest";

const SOURCES = import.meta.glob(
  "/src/components/**/*.{ts,tsx}",
  { eager: true, query: "?raw", import: "default" },
);

const EVENT_PATTERN = /(?:window|document)\.addEventListener\(\s*["'](?:keydown|mousedown|click|pointerdown|resize|scroll)["']/;
const SOURCE_ROOTS = [
  "/src/components/agent-local/",
  "/src/components/file-preview/",
  "/src/components/forecast/",
  "/src/components/internal-browser/",
  "/src/components/terminal/",
  "/src/components/ui/",
];

function scannedSources(): Array<[string, string]> {
  return Object.entries(SOURCES).filter(([path]) => (
    SOURCE_ROOTS.some((root) => path.startsWith(root))
      && !path.includes("/__tests__/")
  ));
}

function directEventSources(): Array<[string, string]> {
  return scannedSources().filter(([, source]) => EVENT_PATTERN.test(source));
}

describe("contrat des listeners d'interface des surfaces", () => {
  it("garde chaque listener DOM par l'activité de surface", () => {
    const failures = directEventSources().flatMap(([path, source]) => {
      if (path.endsWith("/use-browser-surface.ts")) {
        return /if\s*\(\s*!args\.active\b[\s\S]{0,120}return/.test(source)
          ? []
          : [path];
      }
      if (path.endsWith("/use-dialog-keyboard.ts")) return [];
      const addListener = source.search(EVENT_PATTERN);
      const beforeListener = source.slice(0, addListener);
      const guarded = source.includes("useAppSurfaceActive")
        && /if\s*\(\s*!surfaceActive\b[\s\S]{0,120}return/.test(beforeListener);
      return guarded ? [] : [path];
    });

    expect(failures).toEqual([]);
  });

  it("limite l'exception useDialogKeyboard aux réglages et extensions", () => {
    const importers = Object.entries(SOURCES)
      .filter(([, source]) => source.includes("useDialogKeyboard"))
      .map(([path]) => path)
      .filter((path) => !path.endsWith("/use-dialog-keyboard.ts"));

    expect(importers.length).toBeGreaterThan(0);
    expect(importers.every((path) => (
      path.startsWith("/src/components/settings/")
      || path.startsWith("/src/components/extensions/")
    ))).toBe(true);
  });
});
