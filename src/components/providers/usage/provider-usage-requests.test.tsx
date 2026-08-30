/* @vitest-environment jsdom */
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ProviderUsageRequests } from "./provider-usage-requests";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "fr" },
  }),
}));

function metrics(providerRequestId: string | null, finishReason: string | null) {
  return {
    availability: "complete" as const,
    recent: [{
      started_at_ms: 1_800_000_000_000,
      connection_id: "anthropic",
      canonical_provider_id: "anthropic",
      api_format: "anthropic_messages" as const,
      model: "claude-haiku-4-5-20251001",
      routed_provider: null,
      routed_model: null,
      session_id: "session-1",
      request_id: "request-1",
      provider_request_id: providerRequestId,
      finish_reason: finishReason,
      turn: 1,
      attempt: 1,
      workload: "primary" as const,
      origin: "manual_chat" as const,
      status: "completed" as const,
      fast_requested: false,
      service_tier_served: "unknown" as const,
      timing: { headers_ms: 10, first_event_ms: 20, first_useful_ms: 30, total_ms: 40 },
      usage: null,
      usage_complete: false,
    }],
    sessions: [],
  };
}

describe("ProviderUsageRequests", () => {
  it("affiche les métadonnées provider lorsqu'elles existent", () => {
    render(<ProviderUsageRequests metrics={metrics("req_123", "tool_use")} loading={false} />);

    expect(screen.getByText("req_123")).toBeInTheDocument();
    expect(screen.getByText("tool_use")).toBeInTheDocument();
    expect(screen.getByText("providers.usage.requestMetrics.providerRequestId")).toBeInTheDocument();
    expect(screen.getByText("providers.usage.requestMetrics.finishReason")).toBeInTheDocument();
  });

  it("n'affiche aucune ligne vide", () => {
    render(<ProviderUsageRequests metrics={metrics(null, null)} loading={false} />);

    expect(screen.queryByText("providers.usage.requestMetrics.providerRequestId")).not.toBeInTheDocument();
    expect(screen.queryByText("providers.usage.requestMetrics.finishReason")).not.toBeInTheDocument();
  });
});
