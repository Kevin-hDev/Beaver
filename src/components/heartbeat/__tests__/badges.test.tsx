import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RunErrorBadge, ScheduleBadge, StatusBadge } from "../badges";
import type { WakeupDisplayStatus } from "@/types/wakeup";

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key, language: "fr" } }));
vi.mock("@/components/ui/icons", () => ({ Clock: () => null }));

afterEach(cleanup);

describe("StatusBadge", () => {
  it.each(["active", "disabled", "completed", "running", "paused_by_global"] as WakeupDisplayStatus[])(
    "affiche l'état %s fourni par le backend",
    (status) => {
      const { getByText } = render(<StatusBadge status={status} />);
      expect(getByText(`heartbeat.badges.${status}`).className).toContain(`wk-badge-${status}`);
    },
  );
});

describe("RunErrorBadge", () => {
  it("rend visible une session cible disparue", () => {
    const { getByText } = render(<RunErrorBadge run={{
      status: "error", finished_at: "2026-09-11T00:00:00Z", error_code: "target_session_missing",
    }} />);
    expect(getByText("heartbeat.history.errors.targetSessionMissing")).toBeInTheDocument();
  });
});

describe("ScheduleBadge", () => {
  it("affiche les trois formes de planification", () => {
    const { rerender, container } = render(
      <ScheduleBadge schedule={{ kind: "cron", expression: "*/10 * * * *", timezone: "Europe/Paris" }} />,
    );
    expect(container.textContent).toContain("*/10 * * * *");
    rerender(<ScheduleBadge schedule={{ kind: "after_completion", delay_minutes: 20 }} />);
    expect(container.textContent).toContain("wakeupFormat.afterCompletion");
    rerender(<ScheduleBadge schedule={{ kind: "once", local_datetime: "2026-09-12T10:00", timezone: "Europe/Paris" }} />);
    expect(container.textContent).toContain("2026-09-12 10:00");
  });
});
