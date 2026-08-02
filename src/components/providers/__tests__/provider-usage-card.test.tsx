/* @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ProviderUsageSnapshot, UsageAggregate, UsagePeriodId } from "@/types/provider-usage";
import { ProviderUsageCard } from "../usage/provider-usage-card";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, string | number>) => {
      if (values?.name) return `${key}:${values.name}`;
      return values?.value === undefined ? key : `${key}:${values.value}`;
    },
    i18n: { language: "fr" },
  }),
}));
vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn(() => Promise.resolve()) }));

function aggregate(requestCount: number): UsageAggregate {
  return {
    tokens: {
      input_tokens: requestCount * 10,
      output_tokens: requestCount * 5,
      cached_input_tokens: requestCount * 2,
      cache_write_input_tokens: requestCount,
      cache_miss_input_tokens: requestCount * 8,
      reasoning_output_tokens: requestCount,
      total_tokens: requestCount * 15,
    },
    request_count: requestCount,
    usage_request_count: requestCount,
    cache_read_request_count: requestCount,
    cache_write_request_count: requestCount,
    cache_miss_request_count: requestCount,
    cost_usd_micros: requestCount * 100,
    priced_request_count: requestCount,
    exact_cost_request_count: 0,
  };
}

function period(id: UsagePeriodId, count: number) {
  return {
    period: id,
    totals: aggregate(count),
    origins: { manual_chat: aggregate(count), external_channel: aggregate(0), automation: aggregate(0) },
    workloads: { primary: aggregate(count), subagent: aggregate(0), compression: aggregate(0) },
    cost_quality: "estimated" as const,
  };
}

const snapshot: ProviderUsageSnapshot = {
  connection_id: "openrouter",
  canonical_provider_id: "openrouter",
  auth_source: "api",
  availability: "complete",
  windows: [{
    label_code: "key_limit",
    used: 25,
    limit: 100,
    remaining: 75,
    used_percent: 25,
    resets_at: null,
  }],
  balances: [{ label_code: "remaining_credits", amount: "12.5", currency: "USD" }],
  local_periods: [period("today", 1), period("seven_days", 7), period("thirty_days", 30), period("all_time", 40)],
  request_metrics: {
    availability: "complete",
    recent: [{
      started_at_ms: 1_800_000_000_000,
      connection_id: "openrouter",
      canonical_provider_id: "openrouter",
      api_format: "chat_completions",
      model: "openai/gpt-5.6-sol",
      routed_provider: null,
      routed_model: null,
      session_id: "session-1",
      request_id: "request-1",
      turn: 1,
      attempt: 2,
      workload: "primary",
      origin: "manual_chat",
      status: "completed",
      timing: { headers_ms: 120, first_event_ms: 180, first_useful_ms: 220, total_ms: 400 },
      usage: {
        input_tokens: 100,
        output_tokens: 20,
        cached_input_tokens: 80,
        cache_write_input_tokens: 20,
        cache_miss_input_tokens: 20,
        cache_miss_source: "calculated",
        cache_status: "reported",
        reasoning_output_tokens: null,
        total_tokens: 120,
        exact_cost_usd_micros: null,
      },
      usage_complete: true,
    }],
    sessions: [{
      session_id: "session-1",
      attempt_count: 2,
      completed_count: 1,
      usage_complete_count: 1,
      cache_observation_count: 1,
      cache_read_observation_count: 1,
      cache_write_observation_count: 1,
      cache_miss_observation_count: 1,
      cache_read_tokens: 80,
      cache_write_tokens: 20,
      cache_miss_tokens: 20,
      total_duration_ms: 800,
      latest_started_at_ms: 1_800_000_000_000,
    }],
  },
  notice_code: null,
  refreshed_at: 1_800_000_000,
  stale: false,
};

describe("ProviderUsageCard", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset().mockResolvedValue(snapshot);
  });

  it("affiche les limites et sélectionne sept jours par défaut", async () => {
    render(<ProviderUsageCard connectionId="openrouter" siteUrl="https://openrouter.ai" />);
    expect(await screen.findByRole("progressbar")).toHaveAttribute("aria-valuenow", "25");
    expect(screen.getByText("25 / 100")).toBeInTheDocument();
    const row = screen.getByText("providers.usage.requests").parentElement;
    expect(row).not.toBeNull();
    expect(within(row!).getByText("7")).toBeInTheDocument();
  });

  it("affiche le pourcentage restant pour une connexion OAuth", async () => {
    vi.mocked(invoke).mockResolvedValue({
      ...snapshot,
      connection_id: "codex-oauth",
      canonical_provider_id: "openai",
      auth_source: "oauth",
      windows: [{ ...snapshot.windows[0], used: 4, remaining: 96, used_percent: 4 }],
    });
    render(<ProviderUsageCard connectionId="codex-oauth" siteUrl="https://chatgpt.com" />);
    expect(await screen.findByText("providers.usage.remainingPercent:96")).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "96");
  });

  it("sépare les limites générales et celles de Codex Spark", async () => {
    vi.mocked(invoke).mockResolvedValue({
      ...snapshot,
      connection_id: "codex-oauth",
      canonical_provider_id: "openai",
      auth_source: "oauth",
      windows: [
        {
          ...snapshot.windows[0],
          group_code: "general",
          group_name: null,
          used: 4,
          remaining: 96,
          used_percent: 4,
        },
        {
          ...snapshot.windows[0],
          group_code: "codex_bengalfox",
          group_name: "GPT-5.3-Codex-Spark",
          used: 0,
          remaining: 100,
          used_percent: 0,
        },
      ],
    });

    render(<ProviderUsageCard connectionId="codex-oauth" siteUrl="https://chatgpt.com" />);

    expect(await screen.findByText("providers.usage.generalLimitsTitle")).toBeInTheDocument();
    expect(
      screen.getByText("providers.usage.namedLimitsTitle:GPT-5.3-Codex-Spark"),
    ).toBeInTheDocument();
    expect(screen.getAllByRole("progressbar").map((bar) => bar.getAttribute("aria-valuenow")))
      .toEqual(["96", "100"]);
  });

  it("change de période et force une actualisation manuelle", async () => {
    render(<ProviderUsageCard connectionId="openrouter" siteUrl="https://openrouter.ai" />);
    await screen.findByRole("progressbar");
    fireEvent.click(screen.getByText("providers.usage.periods.today"));
    const row = screen.getByText("providers.usage.requests").parentElement;
    expect(within(row!).getByText("1")).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText("providers.usage.refresh"));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("get_provider_usage", {
      connectionId: "openrouter",
      forceRefresh: true,
    }));
  });

  it("affiche les durées et le cache de la dernière requête", async () => {
    render(<ProviderUsageCard connectionId="openrouter" siteUrl="https://openrouter.ai" />);

    expect(await screen.findByText("openai/gpt-5.6-sol")).toBeInTheDocument();
    expect(screen.getByText(/providers\.usage\.requestMetrics\.turn:1/)).toBeInTheDocument();
    expect(screen.getByText("180 ms")).toBeInTheDocument();
    expect(screen.getByText("220 ms")).toBeInTheDocument();
    expect(screen.getByText("400 ms")).toBeInTheDocument();
    expect(screen.getAllByText("80")).toHaveLength(2);
  });

  it("affiche le fournisseur réellement choisi par OpenRouter", async () => {
    vi.mocked(invoke).mockResolvedValue({
      ...snapshot,
      request_metrics: {
        ...snapshot.request_metrics,
        recent: [{
          ...snapshot.request_metrics.recent[0],
          routed_provider: "Google Vertex",
          routed_model: "google/gemini-3.5-pro",
        }],
      },
    });
    render(<ProviderUsageCard connectionId="openrouter" siteUrl="https://openrouter.ai" />);

    expect(await screen.findByText("Google Vertex")).toBeInTheDocument();
    expect(screen.getByText("google/gemini-3.5-pro")).toBeInTheDocument();
  });

  it("conserve les anciens totaux cache positifs comme mesure partielle", async () => {
    const sevenDays = period("seven_days", 7);
    sevenDays.totals.cache_read_request_count = 0;
    vi.mocked(invoke).mockResolvedValue({
      ...snapshot,
      local_periods: snapshot.local_periods.map((item) => (
        item.period === "seven_days" ? sevenDays : item
      )),
    });
    render(<ProviderUsageCard connectionId="openrouter" siteUrl="https://openrouter.ai" />);

    const heading = await screen.findByRole("heading", { name: "providers.usage.localTitle" });
    const section = heading.closest<HTMLElement>("section");
    expect(section).not.toBeNull();
    const label = within(section!).getByText("providers.usage.cachedTokens");
    const row = label.closest<HTMLElement>(".settings-row");
    expect(row).not.toBeNull();
    expect(within(row!).getByText("14")).toBeInTheDocument();
    expect(within(row!).getByText("providers.usage.quality.partial")).toBeInTheDocument();
  });

  it("garde les métriques cache absentes comme inconnues", async () => {
    vi.mocked(invoke).mockResolvedValue({
      ...snapshot,
      request_metrics: {
        ...snapshot.request_metrics,
        recent: [{ ...snapshot.request_metrics.recent[0], usage: null, usage_complete: false }],
        sessions: [{
          ...snapshot.request_metrics.sessions[0],
          cache_observation_count: 0,
          cache_read_observation_count: 0,
          cache_write_observation_count: 0,
          cache_miss_observation_count: 0,
          cache_read_tokens: 0,
          cache_write_tokens: 0,
          cache_miss_tokens: 0,
        }],
      },
    });
    render(<ProviderUsageCard connectionId="openrouter" siteUrl="https://openrouter.ai" />);

    const sessionLabel = await screen.findByText("providers.usage.requestMetrics.latestSession");
    const card = sessionLabel.closest<HTMLElement>(".settings-card");
    expect(card).not.toBeNull();
    expect(within(card!).getAllByText("—").length).toBeGreaterThanOrEqual(6);
  });

  it("n'affiche aucun détail interne si la télémétrie échoue", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("secret-token /private/path"));
    render(<ProviderUsageCard connectionId="openrouter" siteUrl="https://openrouter.ai" />);
    expect(await screen.findByText("providers.usage.remoteUnavailable")).toBeInTheDocument();
    expect(screen.queryByText(/secret-token|private\/path/)).toBeNull();
  });
});
