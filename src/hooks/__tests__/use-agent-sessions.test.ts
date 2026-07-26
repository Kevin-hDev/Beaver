import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useAgentSessions } from "../use-agent-sessions";
import { AGENT_SESSIONS_CHANGED } from "../agent-session-events";
import type { AgentSessionMeta } from "@/types/agent";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

const mockSession: AgentSessionMeta = {
  id: "session-1",
  name: "Test session",
  created_at: "2026-05-09T10:00:00Z",
  model: "llama3",
  provider: "ollama",
  message_count: 3,
};

describe("useAgentSessions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue([mockSession]);
  });

  it("charge les sessions au mount via list_agent_sessions", async () => {
    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(invoke).toHaveBeenCalledWith("list_agent_sessions");
    expect(result.current.sessions).toEqual([mockSession]);
  });

  it("loading passe à false après chargement", async () => {
    const { result } = renderHook(() => useAgentSessions());

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });
  });

  it("sessions est vide si invoke échoue", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("backend KO"));

    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.sessions).toEqual([]);
  });

  it("create appelle invoke create_agent_session puis refresh", async () => {
    const newSession: AgentSessionMeta = { ...mockSession, id: "session-2", name: "Nouvelle" };

    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession])     // list au mount
      .mockResolvedValueOnce(newSession)         // create_agent_session
      .mockResolvedValueOnce([mockSession, newSession]); // list après refresh

    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.create("Nouvelle", "llama3", "ollama");
    });

    expect(invoke).toHaveBeenCalledWith("create_agent_session", {
      name: "Nouvelle",
      model: "llama3",
      provider: "ollama",
      projectId: null,
      reasoningMode: null,
      supportsThinking: null,
    });
    expect(invoke).toHaveBeenCalledWith("list_agent_sessions");
    expect(result.current.sessions).toEqual([mockSession, newSession]);
  });

  it("rename appelle invoke rename_agent_session puis refresh", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession]) // list au mount
      .mockResolvedValueOnce(undefined)     // rename_agent_session
      .mockResolvedValueOnce([{ ...mockSession, name: "Renommé" }]); // list après refresh

    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.rename("session-1", "Renommé");
    });

    expect(invoke).toHaveBeenCalledWith("rename_agent_session", {
      id: "session-1",
      name: "Renommé",
    });
    expect(result.current.sessions[0].name).toBe("Renommé");
  });

  it("remove appelle invoke delete_agent_session puis refresh", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession]) // list au mount
      .mockResolvedValueOnce(undefined)     // delete_agent_session
      .mockResolvedValueOnce([]);           // list après refresh

    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.remove("session-1");
    });

    expect(invoke).toHaveBeenCalledWith("delete_agent_session", { id: "session-1" });
    expect(result.current.sessions).toEqual([]);
  });

  it("archive appelle invoke archive_agent_session puis refresh", async () => {
    const listener = vi.fn();
    window.addEventListener(AGENT_SESSIONS_CHANGED, listener);
    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession])
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([]);

    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.archive("session-1");
    });

    expect(invoke).toHaveBeenCalledWith("archive_agent_session", { id: "session-1" });
    expect(result.current.sessions).toEqual([]);
    expect(listener).toHaveBeenCalledTimes(1);
    window.removeEventListener(AGENT_SESSIONS_CHANGED, listener);
  });

  it("restore appelle invoke restore_agent_session puis refresh", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([mockSession]);

    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.restore("session-1");
    });

    expect(invoke).toHaveBeenCalledWith("restore_agent_session", { id: "session-1" });
    expect(result.current.sessions).toEqual([mockSession]);
  });

  it("updateModel appelle invoke update_session_model puis refresh", async () => {
    const updated: AgentSessionMeta = { ...mockSession, model: "mistral", provider: "ollama" };

    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession]) // list au mount
      .mockResolvedValueOnce(undefined)     // update_session_model
      .mockResolvedValueOnce([updated]);    // list après refresh

    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.updateModel("session-1", "mistral", "ollama");
    });

    expect(invoke).toHaveBeenCalledWith("update_session_model", {
      id: "session-1",
      model: "mistral",
      provider: "ollama",
      reasoningMode: null,
      supportsThinking: null,
    });
    expect(result.current.sessions[0].model).toBe("mistral");
  });

  it("updateReasoning persiste le mode de réflexion puis refresh", async () => {
    const updated: AgentSessionMeta = { ...mockSession, reasoning_mode: "high" };

    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession])
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([updated]);

    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.updateReasoning("session-1", "high");
    });

    expect(invoke).toHaveBeenCalledWith("update_session_reasoning", {
      id: "session-1",
      reasoningMode: "high",
      supportsThinking: null,
    });
    expect(result.current.sessions[0].reasoning_mode).toBe("high");
  });

  it("refresh recharge les sessions depuis le backend", async () => {
    const updated: AgentSessionMeta = { ...mockSession, name: "Après refresh" };

    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession]) // list au mount
      .mockResolvedValueOnce([updated]);    // list après refresh manuel

    const { result } = renderHook(() => useAgentSessions());

    await waitFor(() => expect(result.current.loading).toBe(false));

    await act(async () => {
      await result.current.refresh();
    });

    expect(result.current.sessions[0].name).toBe("Après refresh");
  });

  it("create qui échoue remonte l'erreur et laisse les sessions inchangées", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession])           // list au mount
      .mockRejectedValueOnce(new Error("backend KO")); // create échoue

    const { result } = renderHook(() => useAgentSessions());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await expect(act(() => result.current.create("Test", "llama3"))).rejects.toThrow();
    expect(result.current.sessions).toEqual([mockSession]);
  });

  it("rename qui échoue remonte l'erreur et laisse les sessions inchangées", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession])           // list au mount
      .mockRejectedValueOnce(new Error("backend KO")); // rename échoue

    const { result } = renderHook(() => useAgentSessions());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await expect(act(() => result.current.rename("session-1", "Nouveau nom"))).rejects.toThrow();
    expect(result.current.sessions).toEqual([mockSession]);
  });

  it("remove qui échoue remonte l'erreur et laisse les sessions inchangées", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession])           // list au mount
      .mockRejectedValueOnce(new Error("backend KO")); // remove échoue

    const { result } = renderHook(() => useAgentSessions());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await expect(act(() => result.current.remove("session-1"))).rejects.toThrow();
    expect(result.current.sessions).toEqual([mockSession]);
  });

  it("updateModel qui échoue remonte l'erreur et laisse les sessions inchangées", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([mockSession])           // list au mount
      .mockRejectedValueOnce(new Error("backend KO")); // updateModel échoue

    const { result } = renderHook(() => useAgentSessions());
    await waitFor(() => expect(result.current.loading).toBe(false));

    await expect(
      act(() => result.current.updateModel("session-1", "mistral", "ollama")),
    ).rejects.toThrow();
    expect(result.current.sessions).toEqual([mockSession]);
  });
});
