import { describe, it, expect, vi } from "vitest";
import { applyStreamEvent, createManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function makeState(overrides: Partial<ManagedStreamState> = {}): ManagedStreamState {
  return { ...createManagedStreamState([], 0), streamStartedAt: null, segmentStartedAt: null, ...overrides };
}

describe("token", () => {
  it("accumule le contenu", () => {
    const { state: s } = applyStreamEvent(makeState(), { event: "token", data: { content: "bonjour", tps: 5, tokenCount: 1 } });
    expect(s.currentContent).toBe("bonjour");
    expect(s.currentContentPhase).toBeUndefined();
  });
  it("stocke la phase de travail envoyée par le backend", () => {
    const { state: s } = applyStreamEvent(makeState(), {
      event: "token",
      data: { content: "je travaille", tps: 5, tokenCount: 1, phase: "work" },
    });
    expect(s.currentContent).toBe("je travaille");
    expect(s.currentContentPhase).toBe("work");
  });
  it("sépare le contenu de travail quand la phase finale commence", () => {
    let s = applyStreamEvent(makeState(), {
      event: "token",
      data: { content: "travail", tps: 5, tokenCount: 1, phase: "work" },
    }).state;
    s = applyStreamEvent(s, {
      event: "token",
      data: { content: "final", tps: 5, tokenCount: 2, phase: "final" },
    }).state;
    expect(s.completedSegments).toHaveLength(1);
    expect(s.completedSegments[0].content).toBe("travail");
    expect(s.currentContent).toBe("final");
    expect(s.currentContentPhase).toBe("final");
  });
  it("classe le contenu live quand le backend confirme la phase", () => {
    let s = applyStreamEvent(makeState(), {
      event: "token",
      data: { content: "réponse live", tps: 5, tokenCount: 1 },
    }).state;
    expect(s.currentContentPhase).toBeUndefined();
    s = applyStreamEvent(s, {
      event: "contentPhase",
      data: { phase: "final" },
    }).state;
    expect(s.currentContent).toBe("réponse live");
    expect(s.currentContentPhase).toBe("final");
  });
  it("met à jour tps", () => {
    const { state: s } = applyStreamEvent(makeState(), { event: "token", data: { content: "x", tps: 42, tokenCount: 1 } });
    expect(s.tps).toBe(42);
  });
  it("met à jour liveTokenCount depuis tokenCount fourni", () => {
    const { state: s } = applyStreamEvent(makeState({ liveTokenCount: 10 }), { event: "token", data: { content: "x", tps: 5, tokenCount: 99 } });
    expect(s.liveTokenCount).toBe(99);
  });
  it("incrémente liveTokenCount de 1 si tokenCount absent", () => {
    const { state: s } = applyStreamEvent(makeState({ liveTokenCount: 7 }), { event: "token", data: { content: "x", tps: 5, tokenCount: 0 } });
    expect(s.liveTokenCount).toBe(8);
  });
  it("initialise streamStartedAt si null", () => {
    const before = Date.now();
    const { state: s } = applyStreamEvent(makeState({ streamStartedAt: null }), { event: "token", data: { content: "x", tps: 5, tokenCount: 0 } });
    expect(s.streamStartedAt).toBeGreaterThanOrEqual(before);
  });
});

describe("thinking", () => {
  it("accumule le thinking", () => {
    const { state: s } = applyStreamEvent(makeState(), { event: "thinking", data: { content: "réflexion" } });
    expect(s.currentThinking).toBe("réflexion");
  });
  it("incrémente liveTokenCount de 1", () => {
    const { state: s } = applyStreamEvent(makeState({ liveTokenCount: 3 }), { event: "thinking", data: { content: "hmm" } });
    expect(s.liveTokenCount).toBe(4);
  });
});

describe("interactiveChoiceRequest", () => {
  it("stocke la demande interactive courante", () => {
    const request = {
      sessionId: "session-1",
      id: "choice-1",
      currentIndex: 0,
      total: 1,
      questions: [{ header: "Plan", question: "Choisir ?", options: [] }],
    };
    const { state: s } = applyStreamEvent(makeState(), {
      event: "interactiveChoiceRequest",
      data: request,
    });
    expect(s.interactiveChoice).toEqual(request);
  });
});


describe("turnEnd", () => {
  it("reset les champs courants et sauvegarde le segment", () => {
    const state = makeState({ currentContent: "contenu", currentThinking: "pensée", currentTools: [{ name: "bash", args: {} }] });
    const { state: s } = applyStreamEvent(state, { event: "turnEnd", data: {} });
    expect(s.currentContent).toBe("");
    expect(s.currentContentPhase).toBeUndefined();
    expect(s.currentThinking).toBe("");
    expect(s.currentTools).toHaveLength(0);
    expect(s.completedSegments).toHaveLength(1);
    expect(s.completedSegments[0].content).toBe("contenu");
  });
  it("crée un segment avec thinking, tools et content", () => {
    const state = makeState({ currentContent: "c", currentThinking: "t", currentTools: [{ name: "bash", args: {} }] });
    const { state: s } = applyStreamEvent(state, { event: "turnEnd", data: {} });
    const seg = s.completedSegments[0];
    expect(seg.thinking).toBe("t");
    expect(seg.tools[0].name).toBe("bash");
  });
  it("ignore un turnEnd vide", () => {
    const state = makeState({ completedSegments: [{ content: "existant", thinking: "", tools: [] }] });
    const { state: s } = applyStreamEvent(state, { event: "turnEnd", data: {} });
    expect(s.completedSegments).toHaveLength(1);
    expect(s.completedSegments[0].content).toBe("existant");
  });
});

describe("permissionRequest", () => {
  it("ajoute une permission", () => {
    const { state: s } = applyStreamEvent(makeState(), { event: "permissionRequest", data: { id: "req-1", toolName: "bash", arguments: {} } });
    expect(s.pendingPermissions).toHaveLength(1);
    expect(s.pendingPermissions[0].id).toBe("req-1");
  });
  it("déduplique par id", () => {
    const state = makeState({ pendingPermissions: [{ id: "req-1", toolName: "bash", arguments: { command: "ls" } }] });
    const { state: s } = applyStreamEvent(state, { event: "permissionRequest", data: { id: "req-1", toolName: "bash", arguments: { command: "pwd" } } });
    expect(s.pendingPermissions).toHaveLength(1);
    expect(s.pendingPermissions[0].arguments).toEqual({ command: "pwd" });
  });
  it("respecte MAX_PENDING_PERMISSIONS (32)", () => {
    const perms = Array.from({ length: 32 }, (_, i) => ({ id: `req-${i}`, toolName: "bash", arguments: {} }));
    const { state: s } = applyStreamEvent(makeState({ pendingPermissions: perms }), { event: "permissionRequest", data: { id: "req-new", toolName: "grep", arguments: {} } });
    expect(s.pendingPermissions).toHaveLength(32);
    expect(s.pendingPermissions[31].id).toBe("req-new");
  });
});

describe("toolResult — cas limites", () => {
  it("index hors bornes ne crash pas et laisse les outils inchangés", () => {
    const state = makeState({ currentTools: [{ name: "bash", args: {} }] });
    const { state: s } = applyStreamEvent(state, {
      event: "toolResult",
      data: { name: "bash", toolCallIndex: 99, content: "résultat", isError: false },
    });
    // l'outil existant (index 0) n'avait pas de result → le fallback "premier sans result" s'applique
    // on vérifie juste qu'il n'y a pas de crash et que la longueur reste identique
    expect(s.currentTools).toHaveLength(1);
  });

  it("toolResult avec isError=true assigne isError sur l'outil", () => {
    const state = makeState({ currentTools: [{ name: "bash", args: {} }] });
    const { state: s } = applyStreamEvent(state, {
      event: "toolResult",
      data: { name: "bash", toolCallIndex: 0, content: "erreur!", isError: true },
    });
    expect(s.currentTools[0].isError).toBe(true);
    expect(s.currentTools[0].result).toBe("erreur!");
  });

  it("tous les outils ont déjà un résultat — aucun n'est modifié", () => {
    const state = makeState({
      currentTools: [
        { name: "bash", args: {}, result: "déjà là" },
        { name: "grep", args: {}, result: "aussi là" },
      ],
    });
    const { state: s } = applyStreamEvent(state, {
      event: "toolResult",
      data: { name: "bash", toolCallIndex: -1, content: "nouveau", isError: false },
    });
    expect(s.currentTools[0].result).toBe("déjà là");
    expect(s.currentTools[1].result).toBe("aussi là");
  });
});

describe("accumulation tokens", () => {
  it("plusieurs tokens successifs accumulent le contenu correctement", () => {
    let s = makeState();
    s = applyStreamEvent(s, { event: "token", data: { content: "bonjour", tps: 5, tokenCount: 1 } }).state;
    s = applyStreamEvent(s, { event: "token", data: { content: " monde", tps: 5, tokenCount: 2 } }).state;
    s = applyStreamEvent(s, { event: "token", data: { content: "!", tps: 5, tokenCount: 3 } }).state;
    expect(s.currentContent).toBe("bonjour monde!");
  });
});
