import { useCallback, useMemo, useState } from "react";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FileTreePanel } from "../file-tree-panel";
import type { useFileTree } from "@/hooks/use-file-tree";
import type { FileEntry } from "@/lib/file-tree-filter";

const ROOT: FileEntry = { name: "src", path: "/p/src", is_dir: true, extension: null };
const CHILD: FileEntry = { name: "main.ts", path: "/p/src/main.ts", is_dir: false, extension: "ts" };
const SECOND: FileEntry = { name: "README.md", path: "/p/README.md", is_dir: false, extension: "md" };

function FileTreeHarness() {
  const [expandedPaths, setExpandedPaths] = useState(new Set<string>());
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const childrenMap = useMemo(() => new Map([[ROOT.path, [CHILD]]]), []);

  const toggleExpand = useCallback((path: string) => {
    setExpandedPaths((current) => {
      const next = new Set(current);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
    return Promise.resolve();
  }, []);

  const tree = {
    open: true,
    width: 240,
    resizing: false,
    hasProject: true,
    rootEntries: [ROOT, SECOND],
    childrenMap,
    expandedPaths,
    filter: "",
    loadError: null,
    setFilter: vi.fn(),
    setOpen: vi.fn(),
    toggleOpen: vi.fn(),
    closeTree: vi.fn(),
    toggleExpand,
    startResize: vi.fn(),
  } as unknown as ReturnType<typeof useFileTree>;

  return (
    <>
      <FileTreePanel tree={tree} activePath={selectedPath} onFileSelect={setSelectedPath} />
      <span data-testid="selected">{selectedPath ?? ""}</span>
    </>
  );
}

describe("FileTreePanel keyboard navigation", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((cb) => {
      window.setTimeout(() => cb(0), 0);
      return 0;
    });
  });

  it("ouvre, parcourt et remonte avec les flèches", async () => {
    render(<FileTreeHarness />);
    const root = screen.getByText("src").closest("[data-ft-path]") as HTMLElement;

    root.focus();
    fireEvent.keyDown(root, { key: "ArrowRight" });
    await waitFor(() => expect(screen.getByText("main.ts")).toBeTruthy());

    fireEvent.keyDown(root, { key: "ArrowDown" });
    const child = screen.getByText("main.ts").closest("[data-ft-path]") as HTMLElement;
    await waitFor(() => expect(document.activeElement).toBe(child));
    expect(screen.getByTestId("selected").textContent).toBe(CHILD.path);

    fireEvent.keyDown(child, { key: "ArrowLeft" });
    await waitFor(() => expect(document.activeElement).toBe(root));
  });
});
