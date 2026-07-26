import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useWakeups } from "../use-wakeups";
import type { ScheduledWakeup, WakeupRun, WakeupStatusSummary } from "@/types/wakeup";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

const wakeup: ScheduledWakeup = {
  id: "wakeup-1",
  name: "Daily",
  model: "llama",
  provider: "ollama",
  prompt: "Ping",
  schedule: { kind: "daily", time: "08:00" },
  description: "",
  active: true,
  paused_by_global: false,
  created_at: "2026-05-17T00:00:00Z",
};

const run: WakeupRun = {
  wakeup_id: "wakeup-1",
  scheduled_for: "2026-05-17T08:00:00+02:00",
  fired_at: "2026-05-17T08:00:10Z",
  status: "ok",
  session_id: "session-1",
  tokens: 12,
};

const summary: WakeupStatusSummary = {
  wakeup_id: "wakeup-1",
  next_fire_at: "2026-05-18T08:00:00+02:00",
  last_run: run,
};

function mockInitialLoad() {
  vi.mocked(invoke).mockImplementation((cmd) => {
    if (cmd === "list_wakeups") return Promise.resolve([wakeup]);
    if (cmd === "get_heartbeat_config") return Promise.resolve({ global_paused: false });
    if (cmd === "get_wakeup_status_summaries") return Promise.resolve([summary]);
    if (cmd === "list_wakeup_runs") return Promise.resolve([run]);
    return Promise.resolve(undefined);
  });
}

describe("useWakeups", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInitialLoad();
  });

  it("charge wakeups, config, statuts et historique", async () => {
    const { result } = renderHook(() => useWakeups());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.wakeups).toEqual([wakeup]);
    expect(result.current.globalPaused).toBe(false);
    expect(result.current.runs).toEqual([run]);
    expect(result.current.summaries["wakeup-1"]).toEqual(summary);
  });

  it("écoute les événements de refresh Heartbeat et fichiers", async () => {
    renderHook(() => useWakeups());

    await waitFor(() => {
      expect(listen).toHaveBeenCalledWith("fs:config-changed", expect.any(Function));
    });
    expect(listen).toHaveBeenCalledWith("fs:logs-changed", expect.any(Function));
    expect(listen).toHaveBeenCalledWith("wakeup-completed", expect.any(Function));
    expect(listen).toHaveBeenCalledWith("wakeup-failed", expect.any(Function));
  });
});
