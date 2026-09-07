import { describe, expect, it } from "vitest";

const SOURCES = import.meta.glob(
  "/src/components/**/*.{ts,tsx}",
  { eager: true, query: "?raw", import: "default" },
);

const ALLOWED = new Set([
  "/src/components/ui/app-surface-portal.tsx",
  "/src/components/ui/panel-slots.tsx",
]);
const ROOTS = [
  "/src/components/agent-local/",
  "/src/components/file-preview/",
  "/src/components/forecast/",
  "/src/components/internal-browser/",
  "/src/components/terminal/",
  "/src/components/ui/",
];

describe("contrat des portails de surface", () => {
  it("centralise createPortal dans les deux autorités UI", () => {
    const failures = Object.entries(SOURCES)
      .filter(([path, source]) => (
        ROOTS.some((root) => path.startsWith(root))
        && source.includes("createPortal")
        && !ALLOWED.has(path)
      ))
      .map(([path]) => path);
    expect(failures).toEqual([]);
  });

  it("n'émet pas la virgule de l'ancien argument de createPortal", () => {
    const residuals = Object.entries(SOURCES)
      .filter(([path, source]) => (
        ROOTS.some((root) => path.startsWith(root))
        && /<\/[^>]+>,\s*<\/AppSurfacePortal>/.test(source)
      ))
      .map(([path]) => path);
    expect(residuals).toEqual([]);
  });
});
