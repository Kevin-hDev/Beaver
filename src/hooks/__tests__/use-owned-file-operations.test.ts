import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useOwnedFileOperations } from "../use-owned-file-operations";

describe("useOwnedFileOperations", () => {
  it("ne montre jamais les fichiers de la discussion précédente", () => {
    const { result, rerender } = renderHook(
      ({ ownerKey }) => useOwnedFileOperations(ownerKey),
      { initialProps: { ownerKey: "session:session-a" } },
    );
    const operation = {
      id: "write:a.txt",
      path: "a.txt",
      name: "a.txt",
      type: "write" as const,
      timestamp: "2026-09-01T00:00:00Z",
      additions: 1,
      deletions: 0,
    };

    act(() => result.current.setOperations({ all: [operation], latest: [operation] }));
    expect(result.current.operations.all).toEqual([operation]);

    rerender({ ownerKey: "session:session-b" });
    expect(result.current.operations).toEqual({ all: [], latest: [] });

    rerender({ ownerKey: "session:session-a" });
    expect(result.current.operations.all).toEqual([operation]);
  });

  it("partage les fichiers entre les discussions qui ont le même propriétaire", () => {
    const { result, rerender } = renderHook(
      ({ ownerKey }) => useOwnedFileOperations(ownerKey),
      { initialProps: { ownerKey: "project-a" } },
    );
    const operation = {
      id: "write:shared.txt",
      path: "shared.txt",
      name: "shared.txt",
      type: "write" as const,
      timestamp: "2026-09-01T00:00:00Z",
      additions: 1,
      deletions: 0,
    };

    act(() => result.current.setOperations({ all: [operation], latest: [operation] }));
    rerender({ ownerKey: "project-a" });

    expect(result.current.operations.all).toEqual([operation]);
  });
});
