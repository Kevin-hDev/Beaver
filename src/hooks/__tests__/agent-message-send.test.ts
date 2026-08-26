import { beforeEach, describe, expect, it, vi } from "vitest";
import { persistAgentMessage } from "../agent-message-send";
import { showToast } from "@/lib/toast-emitter";

const invoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

describe("persistAgentMessage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    invoke.mockReset().mockResolvedValue(undefined);
  });

  it("met en file une seule intention et ne relance pas le stream", async () => {
    const doStream = vi.fn();
    const queueStreamMessage = vi.fn().mockResolvedValue("queued");
    await persistAgentMessage({
      sessionId: "session-1", messages: [], text: "Compare",
      skills: [{ id: "local:review", name: "review" }],
      doStream, queueStreamMessage,
    });

    expect(queueStreamMessage).toHaveBeenCalledWith(
      "session-1",
      { content: "Compare", files: [], skills: [{ id: "local:review", name: "review" }] },
      expect.objectContaining({ role: "user", content: "Compare" }),
    );
    expect(doStream).not.toHaveBeenCalled();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("refuse génériquement l'envoi pendant un arrêt sans créer d'optimiste", async () => {
    const doStream = vi.fn();
    const queueStreamMessage = vi.fn().mockResolvedValue("stopping");
    const accepted = await persistAgentMessage({
      sessionId: "session-1", messages: [], text: "Conserve-moi",
      doStream, queueStreamMessage,
    });

    expect(accepted).toBe(false);
    expect(doStream).not.toHaveBeenCalled();
    expect(showToast).toHaveBeenCalledWith(
      "errors.admission.serviceShuttingDown", "error",
    );
  });

  it("démarre avec TurnStart sans persistance frontend ni contenu de skill", async () => {
    const doStream = vi.fn();
    await persistAgentMessage({
      sessionId: "session-1", messages: [], text: "Analyse",
      skills: [{ id: "local:audit", name: "audit" }], doStream,
    });

    expect(doStream).toHaveBeenCalledOnce();
    const turn: unknown = doStream.mock.calls[0]?.[0];
    expect(turn).toEqual({
      type: "new",
      input: {
        content: "Analyse", files: [],
        skills: [{ id: "local:audit", name: "audit" }],
      },
    });
    expect(JSON.stringify(turn)).not.toContain("content de manifeste");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("associe le projet seul puis laisse Rust admettre le user", async () => {
    const doStream = vi.fn();
    await persistAgentMessage({
      sessionId: "session-1", messages: [], text: "Premier message",
      projectId: "project-1", doStream,
    });

    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("update_session_project", {
      id: "session-1", projectId: "project-1",
    });
    expect(doStream).toHaveBeenCalledOnce();
  });

  it("arrête l'envoi si l'association du projet est refusée", async () => {
    invoke.mockRejectedValueOnce(new Error("subagent-read-only"));
    const doStream = vi.fn();
    await persistAgentMessage({
      sessionId: "child-session", messages: [], text: "Message interdit",
      projectId: "project-1", doStream,
    });

    expect(showToast).toHaveBeenCalledWith("errors.admission.subagentReadOnly", "error");
    expect(doStream).not.toHaveBeenCalled();
  });
});
