import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { WakeupHistory } from "../wakeup-history";

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key, language: "fr" } }));

describe("WakeupHistory", () => {
  it("affiche début, fin et erreur stable sans donnée interne", () => {
    const { container } = render(<WakeupHistory runs={[{
      automation_id: "w1", scheduled_for: "2026-05-17T08:00:00Z",
      started_at: "2026-05-17T08:00:10Z", finished_at: "2026-05-17T08:01:00Z",
      status: "error", error_code: "capacity_reached", error: "/private/config.json",
    }]} hasMore={false} onLoadMore={vi.fn()} />);
    expect(container.textContent).toContain("heartbeat.history.started");
    expect(container.textContent).toContain("heartbeat.history.finished");
    expect(screen.getByText("heartbeat.history.errors.capacityReached")).toBeInTheDocument();
    expect(container.textContent).not.toContain("/private/config.json");
  });

  it("demande la page suivante", () => {
    const onLoadMore = vi.fn();
    render(<WakeupHistory runs={[]} hasMore onLoadMore={onLoadMore} />);
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.history.loadMore" }));
    expect(onLoadMore).toHaveBeenCalledOnce();
  });
});
