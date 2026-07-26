import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useGitWatcher } from "../use-git-watcher";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("useGitWatcher", () => {
  it("ordonne start, stop puis le nouveau start lors d'un changement rapide", async () => {
    const firstStart = deferred<void>();
    const calls: string[] = [];
    vi.mocked(invoke).mockImplementation((command, args) => {
      const path = (args as { path: string }).path;
      calls.push(`${command}:${path}`);
      if (command === "start_git_watcher" && path === "/repo") return firstStart.promise;
      return Promise.resolve();
    });

    const view = renderHook(
      ({ path }: { path?: string }) => useGitWatcher(path),
      { initialProps: { path: "/repo" } },
    );
    await waitFor(() => expect(calls).toEqual(["start_git_watcher:/repo"]));

    view.rerender({ path: "/other" });
    expect(calls).toEqual(["start_git_watcher:/repo"]);

    act(() => firstStart.resolve());
    await waitFor(() => expect(calls).toEqual([
      "start_git_watcher:/repo",
      "stop_git_watcher:/repo",
      "start_git_watcher:/other",
    ]));

    view.unmount();
    await waitFor(() => expect(calls[calls.length - 1]).toBe("stop_git_watcher:/other"));
  });
});

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}
