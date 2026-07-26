import { describe, expect, it } from "vitest";
import { isHiddenEntry } from "./file-tree-hidden";

describe("isHiddenEntry", () => {
  it("hides .git", () => expect(isHiddenEntry(".git")).toBe(true));
  it("hides target", () => expect(isHiddenEntry("target")).toBe(true));
  it("hides .DS_Store", () => expect(isHiddenEntry(".DS_Store")).toBe(true));
  it("hides __pycache__", () => expect(isHiddenEntry("__pycache__")).toBe(true));
  it("keeps node_modules visible", () => expect(isHiddenEntry("node_modules")).toBe(false));
  it("keeps .env visible", () => expect(isHiddenEntry(".env")).toBe(false));
  it("keeps src visible", () => expect(isHiddenEntry("src")).toBe(false));
  it("keeps README.md visible", () => expect(isHiddenEntry("README.md")).toBe(false));
});
