import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { usePermissionMode } from "../use-permission-mode";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
vi.mock("@/hooks/use-fs-event", () => ({
  useFsEvent: vi.fn(),
}));

describe("usePermissionMode", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation((command, args) => {
      if (command === "get_agent_settings") return Promise.resolve({ permission_mode: "auto" });
      if (command === "get_session_permission_state") {
        return Promise.resolve({ permission_family: null, permission_mode: "auto" });
      }
      if (command === "set_session_permission_mode") {
        const mode = (args as { mode: "auto" | "manual" | "chat" }).mode;
        return Promise.resolve({
          permission_family: null,
          permission_mode: mode,
        });
      }
      return Promise.resolve(undefined);
    });
  });

  it("mode par défaut est 'auto' après chargement", async () => {
    const { result } = renderHook(() => usePermissionMode());

    await waitFor(() => {
      expect(result.current.loaded).toBe(true);
    });

    expect(result.current.mode).toBe("auto");
  });

  it("loaded passe à true après le premier rendu", async () => {
    const { result } = renderHook(() => usePermissionMode());

    await waitFor(() => {
      expect(result.current.loaded).toBe(true);
    });
  });

  it("change('manual') met à jour le mode", async () => {
    vi.mocked(invoke).mockResolvedValue({ permission_mode: "auto" });
    const { result } = renderHook(() => usePermissionMode());

    await waitFor(() => expect(result.current.loaded).toBe(true));

    await act(async () => {
      await result.current.change("manual");
    });

    expect(result.current.mode).toBe("manual");
  });

  it("change appelle invoke('set_permission_mode', { mode })", async () => {
    const { result } = renderHook(() => usePermissionMode());

    await waitFor(() => expect(result.current.loaded).toBe(true));

    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    await act(async () => {
      await result.current.change("manual");
    });

    expect(invoke).toHaveBeenCalledWith("set_permission_mode", { mode: "manual" });
  });

  it("toggle cycle de chat → manual", async () => {
    const { result } = renderHook(() => usePermissionMode());

    await waitFor(() => expect(result.current.loaded).toBe(true));

    // Passer d'abord en mode chat
    vi.mocked(invoke).mockResolvedValue(undefined);
    await act(async () => {
      await result.current.change("chat");
    });
    expect(result.current.mode).toBe("chat");

    // toggle : chat → manual
    act(() => {
      result.current.toggle();
    });

    await waitFor(() => {
      expect(result.current.mode).toBe("manual");
    });
  });

  it("toggle cycle de manual → auto", async () => {
    const { result } = renderHook(() => usePermissionMode());

    await waitFor(() => expect(result.current.loaded).toBe(true));

    vi.mocked(invoke).mockResolvedValue(undefined);
    await act(async () => {
      await result.current.change("manual");
    });
    expect(result.current.mode).toBe("manual");

    act(() => {
      result.current.toggle();
    });

    await waitFor(() => {
      expect(result.current.mode).toBe("auto");
    });
  });

  it("toggle cycle de auto → chat", async () => {
    const { result } = renderHook(() => usePermissionMode());

    await waitFor(() => expect(result.current.loaded).toBe(true));

    vi.mocked(invoke).mockResolvedValue(undefined);
    await act(async () => {
      await result.current.change("auto");
    });
    expect(result.current.mode).toBe("auto");

    act(() => {
      result.current.toggle();
    });

    await waitFor(() => {
      expect(result.current.mode).toBe("chat");
    });
  });

  it("avec sessionId, le mode est stocké par session", async () => {
    const { result } = renderHook(() => usePermissionMode("session-abc"));

    await waitFor(() => expect(result.current.loaded).toBe(true));

    await act(async () => {
      await result.current.change("manual");
    });

    expect(result.current.mode).toBe("manual");
    expect(result.current.family).toBeNull();
    expect(result.current.availableModes).toEqual(["chat", "manual", "auto"]);
    expect(invoke).toHaveBeenCalledWith("set_session_permission_mode", {
      id: "session-abc",
      mode: "manual",
    });
    expect(invoke).toHaveBeenCalledWith("set_permission_mode", { mode: "manual" });
  });

  it("une session Chatbot ne propose plus les modes outillés", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "get_session_permission_state") {
        return Promise.resolve({ permission_family: "chat", permission_mode: "chat" });
      }
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => usePermissionMode("session-chat"));

    await waitFor(() => expect(result.current.loaded).toBe(true));

    expect(result.current.mode).toBe("chat");
    expect(result.current.availableModes).toEqual(["chat"]);
  });

  it("sans sessionId, utilise defaultMode rechargé depuis le backend", async () => {
    vi.mocked(invoke).mockResolvedValue({ permission_mode: "auto" });
    const { result } = renderHook(() => usePermissionMode());

    await waitFor(() => {
      expect(result.current.loaded).toBe(true);
      expect(result.current.mode).toBe("auto");
    });
  });

  it("erreur invoke ne crash pas (catch silencieux)", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("backend indisponible"));
    const { result } = renderHook(() => usePermissionMode());

    await waitFor(() => expect(result.current.loaded).toBe(true));

    vi.mocked(invoke).mockRejectedValueOnce(new Error("set échoue"));

    await expect(
      act(async () => {
        await result.current.change("manual");
      }),
    ).resolves.not.toThrow();
  });
});
