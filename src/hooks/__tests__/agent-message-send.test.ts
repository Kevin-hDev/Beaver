import { beforeEach, describe, expect, it, vi } from "vitest";
import { persistAgentMessage } from "../agent-message-send";
import { showToast } from "@/lib/toast-emitter";

const invoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

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

  it("traduit le refus lecture seule lors de la persistance du message", async () => {
    invoke.mockRejectedValueOnce(new Error("subagent-read-only"));

    await persistAgentMessage({
      sessionId: "child-session",
      messages: [],
      text: "Message interdit",
      doStream: vi.fn(),
    });

    expect(showToast).toHaveBeenCalledWith("errors.admission.subagentReadOnly", "error");
  });

  it("arrête l'envoi si l'association du projet est refusée", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "get_agent_session") return Promise.resolve({ project_id: null });
      if (command === "save_agent_session") {
        return Promise.reject(new Error("subagent-read-only"));
      }
      return Promise.resolve(undefined);
    });
    const doStream = vi.fn();

    await persistAgentMessage({
      sessionId: "child-session",
      messages: [],
      text: "Message interdit",
      projectId: "project-1",
      doStream,
    });

    expect(invoke).not.toHaveBeenCalledWith("add_messages_to_session", expect.anything());
    expect(doStream).not.toHaveBeenCalled();
  });
});
