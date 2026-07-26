import { describe, it, expect, vi } from "vitest";
import { applyStreamEvent, createManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function makeState(overrides: Partial<ManagedStreamState> = {}): ManagedStreamState {
  return { ...createManagedStreamState([], 0), streamStartedAt: null, segmentStartedAt: null, ...overrides };
}

describe("error", () => {
  it("met à jour state.error avec clé i18n générique", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "message_inconnu" } });
    expect(result.state.error).toBe("errors.streamInterrupted");
  });

  it("utilise la clé i18n connue pour ollama_connection_lost", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "ollama_connection_lost" } });
    expect(result.state.error).toBe("errors.ollamaConnectionLost");
  });

  it("utilise la clé i18n connue pour auth_failed", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "auth_failed" } });
    expect(result.state.error).toBe("errors.authFailed");
  });

  it("utilise les clés i18n connues supplémentaires", () => {
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "rate_limit" },
    }).state.error).toBe("errors.rateLimited");
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "model_not_found" },
    }).state.error).toBe("errors.modelNotFound");
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "ollama_server_error" },
    }).state.error).toBe("errors.ollamaServerError");
  });

  it("traduit les codes sûrs Grok et Kimi sans afficher le corps HTTP", () => {
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "moonshot_membership_unverified" },
    }).state.error).toBe("errors.moonshotMembershipUnverified");
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "xai_subscription_or_credits_required" },
    }).state.error).toBe("errors.xaiSubscriptionOrCreditsRequired");
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "provider_access_unavailable" },
    }).state.error).toBe("errors.providerAccessUnavailable");
  });

  it("traduit les erreurs réseau stables des fournisseurs", () => {
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "provider_connection_failed" },
    }).state.error).toBe("errors.providerConnectionFailed");
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "provider_temporarily_unavailable" },
    }).state.error).toBe("errors.providerTemporarilyUnavailable");
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "provider_request_rejected" },
    }).state.error).toBe("errors.providerRequestRejected");
    expect(applyStreamEvent(makeState(), {
      event: "error", data: { message: "provider_configuration_invalid" },
    }).state.error).toBe("errors.providerConfigurationInvalid");
  });

  it("marque isConnectionError=true quand le flag est présent", () => {
    const result = applyStreamEvent(makeState(), {
      event: "error", data: { message: "err", isConnection: true } as unknown as { message: string },
    });
    expect(result.state.isConnectionError).toBe(true);
  });

  it("finalise le stream même sur erreur", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "crash" } });
    expect(result.state.isStreaming).toBe(false);
  });

  it("conserve le résumé diagnostic séparé du message générique", () => {
    const result = applyStreamEvent(makeState(), {
      event: "error",
      data: {
        message: "crash",
        diagnostic: {
          requestId: "req-1",
          phase: "tool_execution",
          errorType: "stream_closed",
          lastToolName: "write_file",
          safeSummary: "Interruption pendant le tool write_file.",
        },
      },
    });
    expect(result.state.error).toBe("errors.streamInterrupted");
    expect(result.state.diagnosticSummary).toBe("Interruption pendant le tool write_file.");
  });

  it("ne crée pas de message assistant parasite si le contenu est vide", () => {
    const result = applyStreamEvent(makeState({ currentContent: "" }), {
      event: "error", data: { message: "crash" },
    });
    expect(result.assistantMessage).toBeUndefined();
  });

  it("finalise le contenu courant sans perte silencieuse", () => {
    const result = applyStreamEvent(makeState({ currentContent: "réponse partielle" }), {
      event: "error", data: { message: "crash" },
    });
    expect(result.assistantMessage).toBeDefined();
    expect(result.assistantMessage?.content).toBe("réponse partielle");
  });
});
