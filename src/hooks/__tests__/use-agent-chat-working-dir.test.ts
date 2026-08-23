import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useAgentChat } from "../use-agent-chat";
import type { AgentMessage, AgentSession } from "@/types/agent";

type StartStreamMock = (
  sessionId: string,
  model: string,
  provider: string,
  messages: AgentMessage[],
  think?: boolean,
  startState?: Record<string, unknown>,
  workingDir?: string,
) => void | Promise<void>;

const startStream = vi.fn<StartStreamMock>();
const stopStream = vi.fn();
const subscribeToStream = vi.fn(() => () => {});
const getStreamSnapshot = vi.fn(() => null);

const session: AgentSession = {
  id: "session-1",
  name: "Test",
  model: "llama3",
  provider: "ollama",
  thinking_enabled: false,
  fast_mode_enabled: false,
  project_id: "project-1",
  working_dir: "/private/tmp/stale-directory",
  messages: [
    {
      id: "m1",
      role: "user",
      content: "Salut",
      files: [],
      timestamp: "2026-06-24T10:00:00Z",
    },
    {
      id: "m2",
      role: "assistant",
      content: "Bonjour",
      files: [],
      timestamp: "2026-06-24T10:00:01Z",
    },
  ],
  created_at: "2026-06-24T10:00:00Z",
  accumulated_tokens: 12,
};

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../use-agent-stream", () => ({
  useAgentStream: () => ({
    startStream,
    stopStream,
    subscribeToStream,
    getStreamSnapshot,
  }),
}));
vi.mock("../use-gateway-session-updates", () => ({
  listenGatewaySessionUpdates: vi.fn(() => () => {}),
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function expectLastWorkingDir(workingDir: string | undefined) {
  expect(startStream).toHaveBeenLastCalledWith(
    "session-1",
    "llama3",
    "ollama",
    expect.any(Array),
    false,
    expect.any(Object),
    workingDir,
    undefined,
    undefined,
    undefined,
    undefined,
    undefined,
    false,
  );
}

describe("useAgentChat working directory", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation((command: string) => {
      if (command === "get_agent_session") return Promise.resolve(session);
      if (command === "prepare_agent_send") return Promise.resolve({ status: "ready" });
      return Promise.resolve(undefined);
    });
  });

  it("laisse le backend recalculer la racine pour Reload et Edit", async () => {
    const { result } = renderHook(() =>
      useAgentChat("session-1", "llama3", "ollama"),
    );
    await waitFor(() => expect(result.current.sessionLoading).toBe(false));

    await act(async () => {
      await result.current.reload("m2");
    });
    expectLastWorkingDir(undefined);

    await act(async () => {
      await result.current.edit("m1", "Nouvelle demande");
    });
    expectLastWorkingDir(undefined);
  });

  it("transmet encore le projet choisi lors d'un envoi normal", async () => {
    const { result } = renderHook(() =>
      useAgentChat("session-1", "llama3", "ollama"),
    );
    await waitFor(() => expect(result.current.sessionLoading).toBe(false));

    await act(async () => {
      await result.current.sendMessage(
        "Nouvelle demande",
        undefined,
        "/Users/example/project",
        "project-1",
      );
    });

    expectLastWorkingDir("/Users/example/project");
  });
});
