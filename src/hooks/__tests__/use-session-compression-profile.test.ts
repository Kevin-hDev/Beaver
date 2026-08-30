import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";

const fsListener = vi.hoisted(() => ({ callback: null as null | (() => void) }));
const eventListeners = vi.hoisted(() => new Map<string, () => void>());

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((event: string, callback: () => void) => {
    eventListeners.set(event, callback);
    return Promise.resolve(() => eventListeners.delete(event));
  }),
}));
vi.mock("@/hooks/use-fs-event", () => ({
  useFsEvent: (_event: string, callback: () => void) => { fsListener.callback = callback; },
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));

import { useSessionCompressionProfile } from "../use-session-compression-profile";

const profile = { id: "beaver", name: "Beaver", revision: 1 };
const resolved = {
  id: "beaver",
  name: "Beaver",
  source: "global",
  profile_revision: 1,
  global_selection_revision: 2,
  context_window: 96_000,
  band: "compact",
  available: true,
};

beforeEach(() => {
  vi.clearAllMocks();
  eventListeners.clear();
  fsListener.callback = null;
  vi.mocked(invoke).mockImplementation((command) => {
    if (command === "get_compression_profiles") {
      return Promise.resolve({
        global_profile_id: "beaver",
        global_selection_revision: 2,
        profiles: [profile],
      });
    }
    return Promise.resolve(resolved);
  });
});

describe("useSessionCompressionProfile", () => {
  it("charge la liste et l'état effectif depuis les deux autorités backend", async () => {
    const { result } = renderHook(() => useSessionCompressionProfile("session-1"));
    await waitFor(() => expect(result.current.effective?.id).toBe("beaver"));

    expect(result.current.profiles.map((item) => item.name)).toEqual(["Beaver"]);
    expect(result.current.compressionAvailable).toBe(true);
    expect(invoke).toHaveBeenCalledWith("get_session_compression_profile", {
      sessionId: "session-1",
    });
  });

  it("change uniquement la session et conserve l'ancien état si l'écriture échoue", async () => {
    const { result } = renderHook(() => useSessionCompressionProfile("session-1"));
    await waitFor(() => expect(result.current.effective).not.toBeNull());
    vi.mocked(invoke).mockRejectedValueOnce(new Error("failed"));

    await act(async () => {
      expect(await result.current.select("custom")).toBe(false);
    });
    expect(result.current.effective?.id).toBe("beaver");
    expect(invoke).toHaveBeenLastCalledWith("set_session_compression_profile", {
      sessionId: "session-1",
      profileId: "custom",
    });
  });

  it("recalcule après un changement global, de session ou de Modelfile", async () => {
    renderHook(() => useSessionCompressionProfile("session-1"));
    await waitFor(() => expect(eventListeners.has("modelfile-updated")).toBe(true));
    const before = vi.mocked(invoke).mock.calls.filter(
      ([command]) => command === "get_session_compression_profile",
    ).length;

    act(() => {
      fsListener.callback?.();
      eventListeners.get("modelfile-updated")?.();
      window.dispatchEvent(new Event("clgo-agent-sessions-changed"));
    });
    await waitFor(() => expect(vi.mocked(invoke).mock.calls.filter(
      ([command]) => command === "get_session_compression_profile",
    ).length).toBeGreaterThanOrEqual(before + 3));
  });
});
