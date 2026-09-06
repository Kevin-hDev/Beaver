import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installJobFixture } from "@/lib/extension-install-job-fixture.test-support";
import { ExtensionInstallRows } from "./extension-install-rows";

const { tracker } = vi.hoisted(() => ({ tracker: { jobs: [] as unknown[], loading: false, loadError: null as string | null,
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

it("reserves the page tracking area through initial loading, empty and error states", () => {
  tracker.jobs = [];
  tracker.loading = true;
  const view = render(<ExtensionInstallRows page onOpen={vi.fn()} />);
  expect(screen.getByRole("status")).toHaveAttribute("aria-busy", "true");
  expect(screen.getByText("common.loading")).toHaveClass("eij-placeholder");
  tracker.loading = false;
  view.rerender(<ExtensionInstallRows page onOpen={vi.fn()} />);
  expect(screen.getByText("extensionInstalls.empty")).toHaveClass("eij-placeholder");
  tracker.loadError = "extensionInstalls.errors.load";
  view.rerender(<ExtensionInstallRows page onOpen={vi.fn()} />);
  expect(screen.getByRole("alert")).toBeVisible();
  expect(screen.queryByText("extensionInstalls.empty")).toBeNull();
  tracker.loadError = null;
});
