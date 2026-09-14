/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { UpdatesSettings } from "../updates-settings";

const checkAll = vi.fn();
const updateOllamaBinary = vi.fn();
const mocks = vi.hoisted(() => ({ invoke: vi.fn(), showToast: vi.fn() }));
const platform = vi.hoisted(() => ({ isLinux: false }));
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
vi.mock("@/lib/platform", () => ({ get IS_LINUX() { return platform.isLinux; } }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: mocks.showToast }));

describe("UpdatesSettings", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    controller.appUpdate = null;
    controller.ollamaBinaryUpdate = null;
    controller.appDownloading = false;
    controller.ollamaBinaryUpdating = false;
    platform.isLinux = false;
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

  it("associe chaque bouton à son produit sur la même ligne", () => {
    controller.appUpdate = { version: "1.1.8", assetUrl: "https://example.invalid/beaver.dmg" };
    controller.ollamaBinaryUpdate = { currentVersion: "0.32.15", latestVersion: "0.33.1" };
    render(<UpdatesSettings />);

    const beaverRow = screen.getByText("updates.appUpdate").closest(".ups-row");
    const ollamaRow = screen.getByText("updates.ollamaBinaryUpdate").closest(".ups-row");

    expect(beaverRow).toHaveTextContent("Beaverupdates.appUpdatev1.1.8");
    expect(ollamaRow).toHaveTextContent("Ollamaupdates.ollamaBinaryUpdatev0.33.1");
    expect(Array.from(beaverRow!.children).map((child) => child.className)).toEqual([
      "ups-product",
      "ups-action",
      "ups-version",
    ]);
    expect(Array.from(ollamaRow!.children).map((child) => child.className)).toEqual([
      "ups-product",
      "ups-action",
      "ups-version",
    ]);
  });

  it("permet de relancer la recherche depuis cet onglet", () => {
    render(<UpdatesSettings />);
    fireEvent.click(screen.getByText("settings.updates.check"));
    expect(checkAll).toHaveBeenCalledOnce();
  });

  it("remplace l'ancienne progression par l'accès à la fenêtre sur macOS et Windows", async () => {
    mocks.invoke.mockResolvedValue(undefined);
    controller.ollamaBinaryUpdate = { currentVersion: "0.32.15", latestVersion: "0.33.1" };
    controller.ollamaBinaryUpdating = true;
    render(<UpdatesSettings />);

    expect(screen.queryByRole("progressbar")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "updates.window.open" }));
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("show_update_progress_window"));
  });

  it("conserve la progression intégrée uniquement sous Linux", () => {
    platform.isLinux = true;
    controller.ollamaBinaryUpdate = { currentVersion: "0.32.15", latestVersion: "0.33.1" };
    controller.ollamaBinaryUpdating = true;
    render(<UpdatesSettings />);

    expect(screen.getByRole("progressbar")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "updates.window.open" })).toBeNull();
  });

  it("affiche une erreur générique si la fenêtre ne peut pas être rouverte", async () => {
    mocks.invoke.mockRejectedValue(new Error("private path"));
    controller.ollamaBinaryUpdate = { currentVersion: "0.32.15", latestVersion: "0.33.1" };
    controller.ollamaBinaryUpdating = true;
    render(<UpdatesSettings />);

    fireEvent.click(screen.getByRole("button", { name: "updates.window.open" }));
    await waitFor(() => expect(mocks.showToast).toHaveBeenCalledWith("updates.window.openFailed", "error"));
  });
});
