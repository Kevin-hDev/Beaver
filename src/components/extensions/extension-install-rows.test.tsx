import { cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installJobFixture } from "@/lib/extension-install-job-fixture.test-support";
import { ExtensionInstallRows } from "./extension-install-rows";

const { tracker } = vi.hoisted(() => ({ tracker: { jobs: [] as unknown[], loadError: null,
  busyIds: new Set<string>(), errors: {}, action: vi.fn(), refresh: vi.fn() } }));
vi.mock("@/hooks/use-extension-install-jobs", () => ({ useExtensionInstallJobs: () => tracker }));
vi.mock("react-i18next", () => ({ useTranslation: () => ({
  t: (key: string) => key, i18n: { language: "en" },
}) }));
afterEach(cleanup);

it("keeps a pending decision and queued work above old results", () => {
  tracker.jobs = [
    installJobFixture({ id: "old", status: "completed", canCancel: false }),
    installJobFixture({ id: "decision", status: "awaitingConfirmation" }),
    installJobFixture({ id: "queued", status: "queued" }),
    installJobFixture({ id: "recent", status: "cancelled", canCancel: false }),
  ];
  const { container } = render(<ExtensionInstallRows onOpen={vi.fn()} />);
  expect(Array.from(container.querySelectorAll<HTMLElement>("[data-install-job]"),
    row => row.dataset.installJob)).toEqual(["decision", "queued", "recent", "old"]);
});
