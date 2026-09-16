import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ScheduledWakeup } from "@/types/wakeup";
import { WakeupDetails } from "../wakeup-details";

vi.mock("react-i18next", () => ({
  initReactI18next: { type: "3rdParty", init: vi.fn() },
  useTranslation: () => ({ t: (key: string) => key }),
}));

const summary: ScheduledWakeup = {
  id: "extension-wakeup",
  revision: 2,
  name: "Extension follow-up",
  provider: "codex-oauth",
  model: "gpt-5.6-luna",
  target: { mode: "new_session", project_id: null },
  schedule: { kind: "after_completion", delay_minutes: 10 },
  status: "disabled",
  running: false,
  paused_by_global: false,
  next_fire_at: null,
  last_run: null,
  origin: "extension",
  inactive_reason: "approval_required",
};

describe("WakeupDetails extension ownership", () => {
  it("shows provenance and requires an explicit reapproval action", () => {
    const approve = vi.fn();
    render(
      <WakeupDetails
        summary={summary}
        detail={null}
        runs={[]}
        hasMore={false}
        loading={false}
        onBack={vi.fn()}
        onToggle={vi.fn()}
        onApprove={approve}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onLoadMore={vi.fn()}
      />,
    );

    expect(screen.getByText("heartbeat.origins.extension")).toBeInTheDocument();
    expect(screen.getByRole("switch", { name: "heartbeat.toggle" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.approveExtension" }));
    expect(approve).toHaveBeenCalledOnce();
  });
});
