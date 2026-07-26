import { describe, expect, it } from "vitest";
import { planStreamEndArtifacts } from "../stream-end-artifacts";
import type { AgentMessage, StreamMessagePart } from "@/types/agent";

const RUN = "7c8e3a14-8811-4d88-9a54-d234547d8d22";

describe("planStreamEndArtifacts", () => {
  it("masque un groupe tant que son stream reste actif", () => {
    const checkpoint = message("a1", "assistant", "checkpoint");
    expect(planStreamEndArtifacts([checkpoint], true, RUN).size).toBe(0);
  });

  it("place un groupe terminé sous sa vraie réponse finale", () => {
    const checkpoint = message("a1", "assistant", "checkpoint");
    const input = message("u1", "user", "input");
    const final = message("a2", "assistant", "final");
    const planned = planStreamEndArtifacts([checkpoint, input, final], false, RUN);

    expect([...planned.keys()]).toEqual(["a2"]);
    expect(planned.get("a2")?.messages.map((item) => item.id)).toEqual(["a1", "a2"]);
  });

  it("place un stream stoppé ou en erreur sous son dernier élément", () => {
    const checkpoint = message("a1", "assistant", "checkpoint");
    const input = message("u1", "user", "input");
    const planned = planStreamEndArtifacts([checkpoint, input], false, RUN);

    expect([...planned.keys()]).toEqual(["u1"]);
    expect(planned.get("u1")?.messages.map((item) => item.id)).toEqual(["a1"]);
  });

  it("conserve le rendu historique des anciennes sessions", () => {
    const legacy = { ...message("a1", "assistant", "final") };
    delete legacy.stream_run_id;
    delete legacy.stream_part;

    expect([...planStreamEndArtifacts([legacy], false, "").keys()]).toEqual(["a1"]);
  });
});

function message(
  id: string,
  role: AgentMessage["role"],
  part: StreamMessagePart,
): AgentMessage {
  return {
    id,
    role,
    content: id,
    files: [],
    timestamp: "2026-07-12T10:00:00Z",
    stream_run_id: RUN,
    stream_part: part,
  };
}
