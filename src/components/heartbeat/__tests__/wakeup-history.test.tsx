import { cleanup, render } from "@testing-library/react";
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
            error: "Réveil raté",
          },
        ]}
      />,
    );

    expect(container.textContent).toContain("heartbeat.status.missed");
    expect(container.textContent).toContain("Réveil raté");
  });
});
