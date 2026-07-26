import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { checkPreviewFilesExist } from "@/services/file-preview";
import { useSessionFileGroups, useSessionFiles } from "../use-session-files";
import type { AgentMessage, ToolActivityRecord } from "@/types/agent";
import type { ToolActivity } from "../agent-chat-utils";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@/services/file-preview", () => ({
  checkPreviewFilesExist: vi.fn(),
}));

afterEach(() => {
  vi.clearAllMocks();
});

describe("useSessionFiles", () => {
  it("retire les fichiers que le disque ne contient plus", async () => {
    vi.mocked(checkPreviewFilesExist).mockResolvedValue([
      { path: "/repo/keep.ts", exists: true },
      { path: "/repo/deleted.ts", exists: false },
    ]);

    const { result } = renderHook(() => useSessionFiles([
      message("m1", [tool({ summary: "/repo/keep.ts", content: "a" })]),
      message("m2", [tool({ summary: "/repo/deleted.ts", content: "b" })]),
    ], [], [], "/repo"));

    await waitFor(() => {
      expect(result.current.map((operation) => operation.path)).toEqual(["/repo/keep.ts"]);
    });
  });

  it("affiche un outil live seulement quand son résultat est arrivé", async () => {
    vi.mocked(checkPreviewFilesExist).mockResolvedValue([
      { path: "/repo/live.ts", exists: true },
    ]);
    const pending: ToolActivity = { name: "write_file", args: { path: "/repo/live.ts", content: "a" } };
    const done: ToolActivity = { ...pending, result: "ok" };

    const { result, rerender } = renderHook(
      ({ currentTools }) => useSessionFiles([], [], currentTools, "/repo"),
      { initialProps: { currentTools: [pending] } },
    );

    expect(result.current).toEqual([]);

    rerender({ currentTools: [done] });

    await waitFor(() => {
      expect(result.current).toHaveLength(1);
      expect(result.current[0].path).toBe("/repo/live.ts");
    });
  });

  it("masque l'ancienne liste pendant la vérification d'une autre session", async () => {
    let resolveFirst: (value: { path: string; exists: boolean }[]) => void = () => {};
    let resolveSecond: (value: { path: string; exists: boolean }[]) => void = () => {};
    vi.mocked(checkPreviewFilesExist)
      .mockImplementationOnce(() => new Promise((resolve) => { resolveFirst = resolve; }))
      .mockImplementationOnce(() => new Promise((resolve) => { resolveSecond = resolve; }));

    const { result, rerender } = renderHook(
      ({ messages }) => useSessionFiles(messages, [], [], "/repo"),
      { initialProps: { messages: [message("old", [tool({ summary: "/repo/old.ts" })])] } },
    );

    resolveFirst([{ path: "/repo/old.ts", exists: true }]);
    await waitFor(() => {
      expect(result.current.map((operation) => operation.path)).toEqual(["/repo/old.ts"]);
    });

    rerender({ messages: [message("new", [tool({ summary: "/repo/new.ts" })])] });

    await waitFor(() => {
      expect(checkPreviewFilesExist).toHaveBeenCalledTimes(2);
    });
    expect(result.current).toEqual([]);

    resolveSecond([{ path: "/repo/new.ts", exists: true }]);
    await waitFor(() => {
      expect(result.current.map((operation) => operation.path)).toEqual(["/repo/new.ts"]);
    });
  });

  it("garde la dernière exécution modifiante si une requête plus récente ne modifie rien", async () => {
    vi.mocked(checkPreviewFilesExist).mockImplementation((paths) => Promise.resolve(
      paths.map((path) => ({ path, exists: true })),
    ));

    const { result } = renderHook(() => useSessionFileGroups([
      message("write", [tool({ summary: "/repo/changed.ts" })]),
      message("read", [{ name: "read_file", summary: "/repo/changed.ts", result: "ok" }]),
    ], [], [], "/repo"));

    await waitFor(() => {
      expect(result.current.latest.map((operation) => operation.path)).toEqual(["/repo/changed.ts"]);
      expect(result.current.all.map((operation) => operation.path)).toEqual(["/repo/changed.ts"]);
    });
  });

  it("remplace la dernière exécution par les fichiers live modifiés pendant le stream", async () => {
    vi.mocked(checkPreviewFilesExist).mockImplementation((paths) => Promise.resolve(
      paths.map((path) => ({ path, exists: true })),
    ));
    const live: ToolActivity = {
      name: "write_file",
      args: { path: "/repo/live.ts", content: "a\nb" },
      result: "ok",
    };

    const { result } = renderHook(() => useSessionFileGroups([
      message("old", [tool({ summary: "/repo/old.ts" })]),
    ], [], [live], "/repo"));

    await waitFor(() => {
      expect(result.current.latest.map((operation) => operation.path)).toEqual(["/repo/live.ts"]);
      expect(result.current.all.map((operation) => operation.path)).toEqual(["/repo/live.ts", "/repo/old.ts"]);
    });
  });
});

function message(id: string, tools: ToolActivityRecord[]): AgentMessage {
  return {
    id,
    role: "assistant",
    content: "",
    files: [],
    timestamp: "2026-07-02T10:00:00Z",
    tool_activities: tools,
  };
}

function tool(overrides: Partial<ToolActivityRecord>): ToolActivityRecord {
  return {
    name: "write_file",
    summary: "/repo/file.ts",
    content: "x",
    ...overrides,
  };
}
