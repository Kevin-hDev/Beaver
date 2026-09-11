import { describe, expect, it, vi } from "vitest";
import { formatDateTime, formatRunStatus, formatSchedule, formatTarget } from "@/lib/wakeup-format";

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key, language: "fr" },
}));

describe("wakeup format", () => {
  it("formate les trois planifications canoniques", () => {
    expect(formatSchedule({ kind: "once", local_datetime: "2026-09-12T08:00", timezone: "Europe/Paris" }))
      .toBe("2026-09-12 08:00 · Europe/Paris");
    expect(formatSchedule({ kind: "cron", expression: "0 8 * * *", timezone: "Europe/Paris" }))
      .toBe("0 8 * * * · Europe/Paris");
    expect(formatSchedule({ kind: "after_completion", delay_minutes: 10 }))
      .toBe("wakeupFormat.afterCompletion");
  });

  it("formate cible, statut et date absente", () => {
    expect(formatTarget({ mode: "new_session" })).toBe("heartbeat.target.new_session");
    expect(formatTarget({ mode: "resume_session", session_id: "s1" })).toBe("heartbeat.target.resume_session");
    expect(formatRunStatus("interrupted")).toBe("heartbeat.status.interrupted");
    expect(formatDateTime(null)).toBe("heartbeat.status.none");
  });
});
