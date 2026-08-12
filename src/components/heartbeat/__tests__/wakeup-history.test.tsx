import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { WakeupHistory } from "../wakeup-history";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key, language: "fr" },
}));

afterEach(cleanup);

describe("WakeupHistory", () => {
  it("affiche l'historique récent", () => {
    const { container } = render(
      <WakeupHistory
        runs={[
          {
            wakeup_id: "w1",
            scheduled_for: "2026-05-17T08:00:00+02:00",
            fired_at: "2026-05-17T08:00:10Z",
            status: "missed",
            error: "/private/config.json",
          },
        ]}
      />,
    );

    expect(container.textContent).toContain("heartbeat.status.missed");
    expect(container.textContent).toContain("heartbeat.history.errors.failed");
    expect(container.textContent).not.toContain("/private/config.json");
  });

  it("rend la traduction d'un vrai code stable sans exposer le code brut", () => {
    render(
      <WakeupHistory
        runs={[
          {
            wakeup_id: "capacity",
            scheduled_for: "2026-05-17T08:00:00+02:00",
            fired_at: "2026-05-17T08:00:10Z",
            status: "error",
            error_code: "capacity_reached",
          },
        ]}
      />,
    );

    expect(screen.getByText("heartbeat.history.errors.capacityReached")).toBeInTheDocument();
    expect(screen.queryByText("capacity_reached")).not.toBeInTheDocument();
  });
});
