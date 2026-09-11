import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useWakeups } from "../use-wakeups";
import type { ScheduledWakeup, WakeupDetail, WakeupHistoryPage } from "@/types/wakeup";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

const wakeup: ScheduledWakeup = {
  id: "wakeup-1", revision: 1, name: "CI", model: "gpt-5.6-luna",
  provider: "codex-oauth", target: { mode: "new_session", project_id: null },
  schedule: { kind: "after_completion", delay_minutes: 10 }, status: "active",
  running: false, paused_by_global: false, next_fire_at: "2026-09-11T10:00:00Z",
  last_run: null,
};

const detail: WakeupDetail = {
  definition: {
    id: wakeup.id, revision: 1, name: wakeup.name, model: wakeup.model,
    provider: wakeup.provider, target: wakeup.target, schedule: wakeup.schedule,
    status: "active", description: null, prompt: "Vérifie", creator_session_id: null,
    created_at: "2026-09-11T09:00:00Z", anchor_at: "2026-09-11T09:00:00Z",
  },
  next_fire_at: wakeup.next_fire_at,
};

const history: WakeupHistoryPage = { entries: [], next_cursor: "older" };

function mockInitialLoad() {
  vi.mocked(invoke).mockImplementation((command) => {
    if (command === "list_wakeups") return Promise.resolve([wakeup]);
    if (command === "get_heartbeat_config") return Promise.resolve({ global_paused: false });
    if (command === "reconcile_automation_migration") {
      return Promise.resolve({ status: "ready", conflicts: [] });
    }
    if (command === "get_wakeup") return Promise.resolve(detail);
    if (command === "list_wakeup_runs") return Promise.resolve(history);
    return Promise.resolve(undefined);
  });
}

describe("useWakeups", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInitialLoad();
  });

  it("charge les automatisations, la pause et la migration sans reconstruire les statuts", async () => {
    const { result } = renderHook(() => useWakeups());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.wakeups).toEqual([wakeup]);
    expect(result.current.wakeups[0].next_fire_at).toBe("2026-09-11T10:00:00Z");
    expect(result.current.migration?.status).toBe("ready");
  });

  it("charge le détail puis une page d'historique supplémentaire", async () => {
    const { result } = renderHook(() => useWakeups());
    await waitFor(() => expect(result.current.loading).toBe(false));
    await act(() => result.current.loadDetail(wakeup.id));
    expect(result.current.detail).toEqual(detail);
    vi.mocked(invoke).mockResolvedValueOnce({
      entries: [{ automation_id: wakeup.id, scheduled_for: "a", finished_at: "b", status: "ok" }],
      next_cursor: null,
    });
    await act(() => result.current.loadMoreHistory());
    expect(result.current.history.entries).toHaveLength(1);
  });

  it("conserve la liste précédente si un rafraîchissement échoue", async () => {
    const { result } = renderHook(() => useWakeups());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(invoke).mockRejectedValue(new Error("private path"));
    await act(() => result.current.refresh());
    expect(result.current.wakeups).toEqual([wakeup]);
    expect(result.current.error).toBe("store_unavailable");
  });

  it("écoute les événements existants", async () => {
    renderHook(() => useWakeups());
    await waitFor(() => expect(listen).toHaveBeenCalledWith("fs:config-changed", expect.any(Function)));
    expect(listen).toHaveBeenCalledWith("wakeup-completed", expect.any(Function));
    expect(listen).toHaveBeenCalledWith("wakeup-failed", expect.any(Function));
  });

  it("expose l'indisponibilité de l'audit après une mutation bloquée", async () => {
    const { result } = renderHook(() => useWakeups());
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(invoke).mockRejectedValueOnce("audit_unavailable");
    await act(async () => {
      await expect(result.current.create({
        name: "CI", model: "gpt", provider: "codex-oauth", prompt: "Vérifie",
        schedule: { kind: "after_completion", delay_minutes: 10 },
      })).rejects.toBe("audit_unavailable");
    });
    expect(result.current.error).toBe("audit_unavailable");
  });
});
