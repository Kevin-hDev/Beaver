import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useTodos } from "../use-todos";
import { records } from "../agent-stream-records";

let streamHandler: ((event: { payload: unknown }) => void) | null = null;
const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]): Promise<unknown> => invokeMock(...args) as Promise<unknown>,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_event: string, handler: (event: { payload: unknown }) => void) => {
    streamHandler = handler;
    return Promise.resolve(() => {});
  }),
}));

beforeEach(() => {
  records.clear();
  invokeMock.mockReset();
  invokeMock.mockResolvedValue({ todos: [] });
});

describe("useTodos", () => {
  it("charge les todos de session", async () => {
    invokeMock.mockResolvedValue({
      todos: [{ content: "Lire", status: "pending" }],
    });

    const { result } = renderHook(() => useTodos("s1"));

    await waitFor(() => expect(result.current).toHaveLength(1));
    expect(result.current[0].content).toBe("Lire");
  });

  it("met à jour la liste via todoUpdated", async () => {
    const { result } = renderHook(() => useTodos("s1"));
    await waitFor(() => expect(streamHandler).toBeTruthy());

    act(() => {
      streamHandler?.({
        payload: {
          sessionId: "s1",
          event: {
            event: "todoUpdated",
            data: { todos: [{ content: "Tester", status: "completed" }] },
          },
        },
      });
    });

    expect(result.current).toHaveLength(1);
    expect(result.current[0].status).toBe("completed");
  });

  it("alimente plusieurs vues depuis la même projection", async () => {
    const first = renderHook(() => useTodos("shared"));
    const second = renderHook(() => useTodos("shared"));
    await waitFor(() => expect(streamHandler).toBeTruthy());

    act(() => {
      streamHandler?.({
        payload: {
          sessionId: "shared",
          event: {
            event: "todoUpdated",
            data: { todos: [{ content: "Partager", status: "completed" }] },
          },
        },
      });
    });

    expect(first.result.current[0]?.content).toBe("Partager");
    expect(second.result.current[0]?.content).toBe("Partager");
  });

  it("conserve une projection confirmée si le chargement durable échoue", async () => {
    invokeMock.mockRejectedValueOnce(new Error("indisponible"));
    const { result } = renderHook(() => useTodos("s1"));
    await waitFor(() => expect(streamHandler).toBeTruthy());

    act(() => {
      streamHandler?.({
        payload: {
          sessionId: "s1",
          event: {
            event: "todoUpdated",
            data: { todos: [{ content: "Conserver", status: "pending" }] },
          },
        },
      });
    });

    await waitFor(() => expect(result.current[0]?.content).toBe("Conserver"));
  });
});
