/* @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ProviderUsageSnapshot, UsageAggregate, UsagePeriodId } from "@/types/provider-usage";
import { ProviderUsageCard } from "../usage/provider-usage-card";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, string>) => values?.name ? `${key}:${values.name}` : key,
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
      reasoning_output_tokens: requestCount,
      total_tokens: requestCount * 15,
    },
    request_count: requestCount,
    usage_request_count: requestCount,
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
    expect(await screen.findByText("providers.usage.remainingPercent")).toBeInTheDocument();
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

  it("n'affiche aucun détail interne si la télémétrie échoue", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("secret-token /private/path"));
    render(<ProviderUsageCard connectionId="openrouter" siteUrl="https://openrouter.ai" />);
    expect(await screen.findByText("providers.usage.remoteUnavailable")).toBeInTheDocument();
    expect(screen.queryByText(/secret-token|private\/path/)).toBeNull();
  });
});
