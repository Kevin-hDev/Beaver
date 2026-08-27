/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { UpdatesSettings } from "../updates-settings";

const checkAll = vi.fn();
const updateOllamaBinary = vi.fn();
const controller = {
  installedAppVersion: "1.1.7",
  installedOllamaVersion: "0.32.15",
  appUpdate: null as { version: string; assetUrl: string } | null,
  ollamaBinaryUpdate: null as { currentVersion: string; latestVersion: string } | null,
  checking: false,
  appDownloading: false,
  appPercent: 0,
  appCancelling: false,
  ollamaBinaryUpdating: false,
  ollamaBinaryPercent: 0,
  ollamaBinaryCancelling: false,
  binaryBusy: false,
  checkAll,
  downloadAppUpdate: vi.fn(),
  updateOllamaBinary,
  cancelAppUpdate: vi.fn(),
  cancelOllamaBinary: vi.fn(),
};

vi.mock("@/hooks/update-context", () => ({ useUpdates: () => controller }));
vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));

describe("UpdatesSettings", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    controller.appUpdate = null;
    controller.ollamaBinaryUpdate = null;
  });

  it("garde en permanence les versions installées", () => {
    render(<UpdatesSettings />);
    expect(screen.getByText("v1.1.7")).toBeTruthy();
    expect(screen.getByText("v0.32.15")).toBeTruthy();
    expect(screen.queryByText("settings.updates.availableTitle")).toBeNull();
  });

  it("affiche uniquement les mises à jour réellement disponibles", () => {
    controller.ollamaBinaryUpdate = { currentVersion: "0.32.15", latestVersion: "0.33.1" };
    render(<UpdatesSettings />);

    expect(screen.getByText("settings.updates.availableTitle")).toBeTruthy();
    expect(screen.getByText("v0.33.1")).toBeTruthy();
    expect(screen.queryByText("v1.1.8")).toBeNull();
    fireEvent.click(screen.getByText("updates.ollamaBinaryUpdate"));
    expect(updateOllamaBinary).toHaveBeenCalledOnce();
  });

  it("permet de relancer la recherche depuis cet onglet", () => {
    render(<UpdatesSettings />);
    fireEvent.click(screen.getByText("settings.updates.check"));
    expect(checkAll).toHaveBeenCalledOnce();
  });
});
