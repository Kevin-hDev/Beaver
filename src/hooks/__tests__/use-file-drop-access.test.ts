import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useFileDrop } from "../use-file-drop";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  readFile: vi.fn(),
  stat: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/plugin-fs", () => ({
  readFile: mocks.readFile,
  stat: mocks.stat,
}));

describe("useFileDrop access grants", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.stat.mockResolvedValue({ size: 4 });
    mocks.readFile.mockResolvedValue(new Uint8Array([1, 2, 3, 4]));
  });

  it("enregistre le chemin avant de lire le fichier", async () => {
    const accessGrant = `v1.${"a".repeat(64)}`;
    mocks.invoke.mockResolvedValue([{
      path: "/tmp/photo.png",
      size: 4,
      access_grant: accessGrant,
    }]);
    const { result } = renderHook(() => useFileDrop());

    await act(async () => result.current.addByPaths(["/tmp/photo.png"]));

    expect(mocks.invoke).toHaveBeenCalledWith("register_attachment_paths", {
      paths: ["/tmp/photo.png"],
    });
    expect(mocks.invoke).toHaveBeenCalledBefore(mocks.readFile);
    expect(result.current.files[0].accessGrant).toBe(accessGrant);
  });

  it("ne lit rien lorsque Rust refuse le chemin", async () => {
    mocks.invoke.mockRejectedValue(new Error("denied"));
    const { result } = renderHook(() => useFileDrop());

    await act(async () => result.current.addByPaths(["/tmp/private.txt"]));

    expect(mocks.readFile).not.toHaveBeenCalled();
    expect(result.current.files).toEqual([]);
    expect(result.current.error).not.toBeNull();
  });
});
