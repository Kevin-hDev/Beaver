import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createMountCoordinator } from "../mount-coordinator";

describe("createMountCoordinator", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockImplementation((command) => Promise.resolve(
      command === "begin_extension_ui_load" ? [1, 2, 3] : undefined
    ));
  });

  it("serializes journals and acknowledges only after the React commit", async () => {
    const coordinator = createMountCoordinator();
    const first = await coordinator.prepare("1:first", "com.example.first", 1);
    const secondPromise = coordinator.prepare("1:second", "com.example.second", 1);

    expect(commands()).toEqual([
      "begin_extension_ui_load",
      "advance_extension_ui_load",
    ]);
    await first.commit();
    const second = await secondPromise;
    expect(commands()).toEqual([
      "begin_extension_ui_load",
      "advance_extension_ui_load",
      "acknowledge_extension_ui_load",
      "begin_extension_ui_load",
      "advance_extension_ui_load",
    ]);
    await second.commit();
    const allCommands = commands();
    expect(allCommands[allCommands.length - 1]).toBe("acknowledge_extension_ui_load");
  });

  it("journals a contribution again when its catalog revision changes", async () => {
    const coordinator = createMountCoordinator();
    const first = await coordinator.prepare("1:com.example.ui:panel", "com.example.ui", 1);
    await first.commit();
    const second = await coordinator.prepare("2:com.example.ui:panel", "com.example.ui", 1);
    await second.commit();

    expect(commands().filter((command) =>
      command === "acknowledge_extension_ui_load")).toHaveLength(2);
  });

  it("deduplicates the same contribution and stops after a failure", async () => {
    const coordinator = createMountCoordinator();
    const permit = await coordinator.prepare("1:same", "com.example.same", 1);
    const duplicate = coordinator.prepare("1:same", "com.example.same", 1);
    await permit.commit();
    await duplicate;
    expect(commands().filter((command) => command === "begin_extension_ui_load")).toHaveLength(1);

    vi.mocked(invoke).mockRejectedValueOnce(new Error("internal"));
    await expect(coordinator.prepare("1:failed", "com.example.failed", 1)).rejects.toThrow();
    const queued = coordinator.prepare("1:queued", "com.example.queued", 1);
    void queued.catch(() => {});
    await Promise.resolve();
    expect(commands().filter((command) => command === "begin_extension_ui_load")).toHaveLength(2);
  });
});

function commands(): string[] {
  return vi.mocked(invoke).mock.calls.map(([command]) => command);
}
