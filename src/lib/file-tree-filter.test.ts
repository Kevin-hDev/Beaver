import { describe, expect, it } from "vitest";
import { filterTree } from "./file-tree-filter";
import type { FileEntry } from "./file-tree-filter";

const TREE: FileEntry[] = [
  { name: "src", path: "/p/src", is_dir: true, extension: null },
  { name: "docs", path: "/p/docs", is_dir: true, extension: null },
  { name: "README.md", path: "/p/README.md", is_dir: false, extension: "md" },
  { name: "main.rs", path: "/p/main.rs", is_dir: false, extension: "rs" },
];

const CHILDREN = new Map<string, FileEntry[]>([
  ["/p/src", [
    { name: "lib.rs", path: "/p/src/lib.rs", is_dir: false, extension: "rs" },
    { name: "utils", path: "/p/src/utils", is_dir: true, extension: null },
  ]],
  ["/p/src/utils", [
    { name: "helpers.rs", path: "/p/src/utils/helpers.rs", is_dir: false, extension: "rs" },
  ]],
]);

describe("filterTree", () => {
  it("returns all entries on empty query", () => {
    const result = filterTree(TREE, CHILDREN, "");
    expect(result.entries).toEqual(TREE);
    expect(result.expandedPaths.size).toBe(0);
  });

  it("matches partial case-insensitive", () => {
    const result = filterTree(TREE, CHILDREN, "read");
    expect(result.entries.map((e) => e.name)).toContain("README.md");
  });

  it("auto-expands parents when child matches", () => {
    const result = filterTree(TREE, CHILDREN, "helpers");
    expect(result.expandedPaths.has("/p/src")).toBe(true);
    expect(result.expandedPaths.has("/p/src/utils")).toBe(true);
  });

  it("includes parent dirs that contain matches", () => {
    const result = filterTree(TREE, CHILDREN, "lib.rs");
    expect(result.entries.map((e) => e.name)).toContain("src");
  });

  it("handles accented characters", () => {
    const entries: FileEntry[] = [
      { name: "résumé.md", path: "/p/résumé.md", is_dir: false, extension: "md" },
    ];
    const result = filterTree(entries, new Map(), "résu");
    expect(result.entries).toHaveLength(1);
  });

  it("handles spaces in query", () => {
    const entries: FileEntry[] = [
      { name: "my file.ts", path: "/p/my file.ts", is_dir: false, extension: "ts" },
    ];
    const result = filterTree(entries, new Map(), "my fi");
    expect(result.entries).toHaveLength(1);
  });
});
