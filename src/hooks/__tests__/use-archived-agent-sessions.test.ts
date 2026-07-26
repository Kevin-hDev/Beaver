import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useArchivedAgentSessions } from "../use-archived-agent-sessions";
import type { AgentSessionMeta } from "@/types/agent";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const archivedSession: AgentSessionMeta = {
  id: "session-1",
  name: "Archived",
  created_at: "2026-05-09T10:00:00Z",
  updated_at: "2026-05-10T10:00:00Z",
  archived_at: "2026-05-11T10:00:00Z",
  model: "llama3",
  provider: "ollama",
  message_count: 3,
};

describe("useArchivedAgentSessions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue([archivedSession]);
  });

  it("charge les sessions archivées au mount", async () => {
    const { result } = renderHook(() => useArchivedAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(invoke).toHaveBeenCalledWith("list_archived_agent_sessions");
    expect(result.current.sessions).toEqual([archivedSession]);
  });

  it("restore désarchive puis recharge", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([archivedSession])
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([]);

    const { result } = renderHook(() => useArchivedAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.restore("session-1");
    });

    expect(invoke).toHaveBeenCalledWith("restore_agent_session", { id: "session-1" });
    expect(result.current.sessions).toEqual([]);
  });

  it("remove supprime définitivement puis recharge", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([archivedSession])
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([]);

    const { result } = renderHook(() => useArchivedAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.remove("session-1");
    });

    expect(invoke).toHaveBeenCalledWith("delete_agent_session", { id: "session-1" });
    expect(result.current.sessions).toEqual([]);
  });
});
