import { beforeEach, describe, expect, it, vi } from "vitest";
import { persistAgentMessage } from "../agent-message-send";

const invoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("persistAgentMessage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("ne relance pas le stream lorsque le message rejoint le run actif", async () => {
    const doStream = vi.fn();
    const queueStreamMessage = vi.fn().mockResolvedValue(true);

    await persistAgentMessage({
      sessionId: "session-1",
      messages: [{
        id: "u1", role: "user", content: "Question", files: [],
        timestamp: "2026-07-12T10:00:00Z",
      }],
      text: "Ajoute une comparaison",
      doStream,
      queueStreamMessage,
    });

    expect(queueStreamMessage).toHaveBeenCalledOnce();
    expect(doStream).not.toHaveBeenCalled();
    expect(invoke).not.toHaveBeenCalledWith("add_messages_to_session", expect.anything());
  });

  it("ne démarre pas l'agent si le premier message n'a pas été enregistré", async () => {
    invoke.mockRejectedValueOnce(new Error("save failed"));
    const doStream = vi.fn();

    await persistAgentMessage({
      sessionId: "session-1",
      messages: [],
      text: "Crée un rapport",
      doStream,
    });

    expect(doStream).not.toHaveBeenCalled();
  });
});
