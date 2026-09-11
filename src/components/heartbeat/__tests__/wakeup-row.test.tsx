import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { WakeupRow } from "../wakeup-row";
import type { ScheduledWakeup } from "@/types/wakeup";

vi.mock("react-i18next", () => ({
  initReactI18next: { type: "3rdParty", init: vi.fn() },
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("WakeupRow", () => {
  it("nomme explicitement la date de la prochaine exécution", () => {
    const wakeup: ScheduledWakeup = {
      id: "wake-1",
      revision: 1,
      name: "CI",
      provider: "codex-oauth",
      model: "gpt-5.6-luna",
      target: { mode: "new_session", project_id: null },
      schedule: { kind: "cron", expression: "*/5 * * * *", timezone: "UTC" },
      status: "active",
      running: false,
      paused_by_global: false,
      next_fire_at: "2026-09-11T10:05:00Z",
      last_run: null,
    };

    render(<WakeupRow wakeup={wakeup} onClick={vi.fn()} />);

    expect(screen.getByText(/heartbeat\.fields\.nextFire/)).toBeInTheDocument();
  });
});
