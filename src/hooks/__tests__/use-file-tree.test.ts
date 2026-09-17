import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useFileTree } from "../use-file-tree";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));

describe("useFileTree", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.invoke.mockImplementation((command: string) => (
      command === "list_directory" ? Promise.resolve([]) : Promise.resolve(undefined)
    ));
  });

  it("démarre et arrête le watcher depuis l'ouverture contrôlée", async () => {
    const { rerender } = renderHook(
      ({ open }) => useFileTree("session-1", "/project", open),
      { initialProps: { open: false } },
    );

    expect(mocks.invoke).not.toHaveBeenCalledWith("watch_project_directory", expect.anything());

    rerender({ open: true });
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "watch_project_directory", { path: "/project" },
    ));

    rerender({ open: false });
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith(
      "unwatch_project_directory",
    ));
  });
});
