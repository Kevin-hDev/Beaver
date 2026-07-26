import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useAgentChat } from "../use-agent-chat";
import type { AgentMessage, AgentSession, FileAttachment } from "@/types/agent";

type StartStreamMock = (
  sessionId: string,
  model: string,
  provider: string,
  messages: AgentMessage[],
) => void | Promise<void>;

const startStream = vi.fn<StartStreamMock>();
const stopStream = vi.fn();
const subscribeToStream = vi.fn(() => () => {});
const getStreamSnapshot = vi.fn(() => null);

interface TruncatePayload {
  sessionId: string;
  messageId: string;
  replacement: AgentMessage | null;
}

let lastStreamMessages: AgentMessage[] | null = null;
let lastTruncatePayload: TruncatePayload | null = null;
let prepareResult: Record<string, unknown> = { status: "ready" };

function mockSessionInvoke(currentSession: AgentSession) {
  vi.mocked(invoke).mockImplementation((command: string, payload?: unknown) => {
    if (command === "get_agent_session") return Promise.resolve(currentSession);
    if (command === "prepare_agent_send") return Promise.resolve(prepareResult);
    if (command === "resolve_missing_session_directory") return Promise.resolve("/tmp");
    if (command === "truncate_and_replace_at") {
      lastTruncatePayload = payload as TruncatePayload;
    }
    return Promise.resolve(undefined);
  });
}

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../use-agent-stream", () => ({
  useAgentStream: () => ({ startStream, stopStream, subscribeToStream, getStreamSnapshot }),
}));
vi.mock("../use-gateway-session-updates", () => ({
  listenGatewaySessionUpdates: vi.fn(() => () => {}),
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

const session: AgentSession = {
  id: "session-1",
  name: "Test",
  model: "llama3",
  provider: "ollama",
  thinking_enabled: false,
  messages: [
    { id: "m1", role: "user", content: "Salut", files: [], timestamp: "2026-06-24T10:00:00Z" },
    { id: "m2", role: "assistant", content: "Bonjour", files: [], timestamp: "2026-06-24T10:00:01Z" },
  ],
  created_at: "2026-06-24T10:00:00Z",
  accumulated_tokens: 12,
};

describe("useAgentChat", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    lastStreamMessages = null;
    lastTruncatePayload = null;
    prepareResult = { status: "ready" };
    startStream.mockImplementation((_sessionId, _model, _provider, messages) => {
      lastStreamMessages = messages;
    });
    mockSessionInvoke(session);
  });

  it("reload utilise truncate_and_replace_at sans ancienne commande truncate_session_at", async () => {
    const { result } = renderHook(() => useAgentChat("session-1", "llama3", "ollama"));
    await waitFor(() => expect(result.current.sessionLoading).toBe(false));

    await act(async () => {
      await result.current.reload("m2");
    });

    expect(invoke).toHaveBeenCalledWith("truncate_and_replace_at", {
      sessionId: "session-1",
      messageId: "m2",
      replacement: null,
    });
    expect(invoke).not.toHaveBeenCalledWith("truncate_session_at", expect.anything());
    expect(startStream).toHaveBeenCalled();
  });

  it("transmet le support vision du modèle sélectionné au stream", async () => {
    const { result } = renderHook(() =>
      useAgentChat(
        "session-1",
        "google/gemma-4-31b-it",
        "openrouter",
        undefined,
        true,
        true,
        true,
        "auto",
      ),
    );
    await waitFor(() => expect(result.current.sessionLoading).toBe(false));

    await act(async () => {
      await result.current.sendMessage("décris l'image");
    });

    expect(startStream).toHaveBeenLastCalledWith(
      "session-1",
      "google/gemma-4-31b-it",
      "openrouter",
      expect.any(Array),
      true,
      expect.any(Object),
      undefined,
      true,
      true,
      true,
      "auto",
      undefined,
      false,
    );
  });

  it("rafraîchit le verrou des modes après le démarrage accepté", async () => {
    const onStreamStarted = vi.fn();
    const { result } = renderHook(() => useAgentChat(
      "session-1", "llama3", "ollama", undefined,
      true, true, false, "auto", "manual", onStreamStarted,
    ));
    await waitFor(() => expect(result.current.sessionLoading).toBe(false));

    await act(async () => {
      await result.current.sendMessage("Premier message");
    });

    expect(onStreamStarted).toHaveBeenCalledOnce();
  });

  it("conserve les fichiers quand un message utilisateur est édité", async () => {
    const imageFile: FileAttachment = {
      name: "logo.png",
      path: "/tmp/logo.png",
      mime_type: "image/png",
      size: 123,
      thumbnail: "data:image/png;base64,abc",
    };
    mockSessionInvoke({
      ...session,
      messages: [{ ...session.messages[0], files: [imageFile] }, session.messages[1]],
    });
    const { result } = renderHook(() => useAgentChat("session-1", "llama3", "ollama"));
    await waitFor(() => expect(result.current.sessionLoading).toBe(false));

    await act(async () => {
      await result.current.edit("m1", "Décris mieux cette image");
    });

    expect(lastTruncatePayload?.sessionId).toBe("session-1");
    expect(lastTruncatePayload?.messageId).toBe("m1");
    expect(lastTruncatePayload?.replacement?.role).toBe("user");
    expect(lastTruncatePayload?.replacement?.content).toBe("Décris mieux cette image");
    expect(lastTruncatePayload?.replacement?.files).toEqual([imageFile]);
    expect(lastStreamMessages?.[0]?.files).toEqual([imageFile]);
  });

  it("diffère l'envoi puis le rejoue une seule fois après Switcher", async () => {
    prepareResult = {
      status: "missing",
      missing_path: "/tmp/gone/project",
      nearest_parent: "/tmp",
    };
    const { result } = renderHook(() => useAgentChat("session-1", "llama3", "ollama"));
    await waitFor(() => expect(result.current.sessionLoading).toBe(false));

    await act(async () => {
      await result.current.sendMessage("Message différé");
    });
    expect(result.current.missingDirectory?.missing_path).toBe("/tmp/gone/project");
    expect(startStream).not.toHaveBeenCalled();
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "add_messages_to_session")).toHaveLength(0);

    await act(async () => {
      await result.current.resolveMissingDirectory("switch");
    });
    expect(startStream).toHaveBeenCalledTimes(1);
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "add_messages_to_session")).toHaveLength(1);
  });
});
