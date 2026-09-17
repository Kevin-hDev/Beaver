import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { checkPreviewFilesExist } from "@/services/file-preview";
import { useSessionFileGroups } from "../use-session-files";
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

describe("useSessionFileGroups", () => {
  it("retire les fichiers que le disque ne contient plus", async () => {
    vi.mocked(checkPreviewFilesExist).mockResolvedValue([
      { path: "/repo/keep.ts", exists: true },
      { path: "/repo/deleted.ts", exists: false },
    ]);

    const { result } = renderHook(() => useSessionFileGroups([
      message("m1", [tool({ summary: "/repo/keep.ts", content: "a" })]),
      message("m2", [tool({ summary: "/repo/deleted.ts", content: "b" })]),
    ], [], [], "/repo"));

    await waitFor(() => {
      expect(result.current.all.map((operation) => operation.path)).toEqual(["/repo/keep.ts"]);
    });
  });

  it("affiche un outil live seulement quand son résultat est arrivé", async () => {
    vi.mocked(checkPreviewFilesExist).mockResolvedValue([
      { path: "/repo/live.ts", exists: true },
    ]);
    const pending: ToolActivity = { name: "write_file", args: { path: "/repo/live.ts", content: "a" } };
    const done: ToolActivity = { ...pending, result: "ok" };

    const { result, rerender } = renderHook(
      ({ currentTools }) => useSessionFileGroups([], [], currentTools, "/repo"),
      { initialProps: { currentTools: [pending] } },
    );

    expect(result.current.all).toEqual([]);

    rerender({ currentTools: [done] });

    await waitFor(() => {
      expect(result.current.all).toHaveLength(1);
      expect(result.current.all[0].path).toBe("/repo/live.ts");
    });
  });

  it("masque l'ancienne liste pendant la vérification d'une autre session", async () => {
    let resolveFirst: (value: { path: string; exists: boolean }[]) => void = () => {};
    let resolveSecond: (value: { path: string; exists: boolean }[]) => void = () => {};
    vi.mocked(checkPreviewFilesExist)
      .mockImplementationOnce(() => new Promise((resolve) => { resolveFirst = resolve; }))
      .mockImplementationOnce(() => new Promise((resolve) => { resolveSecond = resolve; }));

    const { result, rerender } = renderHook(
      ({ messages }) => useSessionFileGroups(messages, [], [], "/repo"),
      { initialProps: { messages: [message("old", [tool({ summary: "/repo/old.ts" })])] } },
    );

    resolveFirst([{ path: "/repo/old.ts", exists: true }]);
    await waitFor(() => {
      expect(result.current.all.map((operation) => operation.path)).toEqual(["/repo/old.ts"]);
    });

    rerender({ messages: [message("new", [tool({ summary: "/repo/new.ts" })])] });

    await waitFor(() => {
      expect(checkPreviewFilesExist).toHaveBeenCalledTimes(2);
    });
    expect(result.current.all).toEqual([]);

    resolveSecond([{ path: "/repo/new.ts", exists: true }]);
    await waitFor(() => {
      expect(result.current.all.map((operation) => operation.path)).toEqual(["/repo/new.ts"]);
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

  it("reconstruit les fichiers des conversations enregistrées sous forme d'appels d'outils", async () => {
    vi.mocked(checkPreviewFilesExist).mockImplementation((paths) => Promise.resolve(
      paths.map((path) => ({ path, exists: true })),
    ));
    const messages = [
      ...savedWriteTurn("first", "/repo/first.ts", "first"),
      ...savedWriteTurn("second", "/repo/second.ts", "second"),
    ];

    const { result } = renderHook(() => useSessionFileGroups(messages, [], [], "/repo"));

    await waitFor(() => {
      expect(result.current.all.map((operation) => operation.path)).toEqual([
        "/repo/second.ts",
        "/repo/first.ts",
      ]);
      expect(result.current.latest.map((operation) => operation.path)).toEqual([
        "/repo/second.ts",
      ]);
    });
  });
});

function savedWriteTurn(id: string, path: string, content: string): AgentMessage[] {
  return [
    {
      id: `${id}-user`,
      role: "user",
      content: "Crée ce fichier.",
      files: [],
      timestamp: "2026-09-11T09:59:59Z",
    },
    {
      id: `${id}-assistant-tool`,
      role: "assistant",
      content: "",
      files: [],
      timestamp: "2026-09-11T10:00:00Z",
      tool_calls: [{ id, function: { name: "write_file", arguments: { path, content } } }],
    },
    {
      id: `${id}-tool-result`,
      role: "tool",
      content: `Écrit: ${path}`,
      files: [],
      timestamp: "2026-09-11T10:00:01Z",
      tool_name: "write_file",
      tool_call_id: id,
    },
    {
      id: `${id}-assistant-final`,
      role: "assistant",
      content: "Terminé.",
      files: [],
      timestamp: "2026-09-11T10:00:02Z",
    },
  ];
}

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
