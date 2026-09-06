import { describe, expect, it } from "vitest";
import { INSTALL_JOB_LIMITS } from "@/types/extension-install-jobs.generated";
import { parseInstallJob, parseInstallJobsSnapshot } from "./extension-install-job-parser";
import { installJobFixture, installSnapshotFixture } from "./extension-install-job-fixture.test-support";
import { installJobErrorKey } from "./extension-install-job-errors";

describe("installation IPC validation", () => {
  it("accepts the generated backend projection", () => {
    const snapshot = installSnapshotFixture();
    expect(parseInstallJobsSnapshot(snapshot)).toEqual(snapshot);
  });
  it.each([
    { occupiedBytes: -1 }, { downloadedBytes: Number.NaN }, { revision: Number.MAX_SAFE_INTEGER + 1 },
    { id: "../private" }, { displayName: "a\nprivate" }, { status: "resumed" }, { source: "/private" },
    { downloadedBytes: 20, downloadTotalBytes: 10 }, { canResume: true }, { errorCode: "Error at /private/path" },
    { confirmationId: "10000000-0000-4000-8000-000000000001" }, { canCancel: "yes" },
  ])("rejects malformed or private fields %j", change => {
    expect(() => parseInstallJob({ ...installJobFixture(), ...change })).toThrow();
  });
  it("bounds snapshots, rejects duplicates and impossible revisions", () => {
    const job = installJobFixture();
    expect(() => parseInstallJobsSnapshot(installSnapshotFixture(1, [job, job]))).toThrow();
    expect(() => parseInstallJobsSnapshot(installSnapshotFixture(0, [job]))).toThrow();
    expect(() => parseInstallJobsSnapshot(installSnapshotFixture(1, Array.from({ length: INSTALL_JOB_LIMITS.active + INSTALL_JOB_LIMITS.recent + 1 }, () => job)))).toThrow();
  });
  it("requires a real waiting job as the queue blocker", () => {
    const waiting = installJobFixture({ status: "awaitingConfirmation", confirmationId: "20000000-0000-4000-8000-000000000001" });
    const queued = installJobFixture({ id: "30000000-0000-4000-8000-000000000001", status: "queued", queueBlocker: { kind: "confirmation", jobId: waiting.id } });
    expect(parseInstallJobsSnapshot(installSnapshotFixture(1, [waiting, queued])).jobs).toHaveLength(2);
    expect(() => parseInstallJobsSnapshot(installSnapshotFixture(1, [queued]))).toThrow();
  });
  it("maps safe error codes and never displays raw diagnostics", () => {
    expect(installJobErrorKey("extension-install-insufficient-space")).toBe("extensionInstalls.errors.insufficientSpace");
    expect(installJobErrorKey("at /private/source token=secret")).toBe("extensionInstalls.errors.action");
    expect(installJobErrorKey("constructor")).toBe("extensionInstalls.errors.action");
  });
});
