/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { UpdateNotifications } from "../update-notifications";

vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn(() => Promise.resolve()) }));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "fr" },
    t: (key: string, opts?: Record<string, string>) => {
      if (key === "updates.version") return `Version ${opts?.version ?? ""}`;
      if (key === "updates.releaseNotesTitle") return `Notes ${opts?.version ?? ""}`;
      return key;
    },
  }),
}));

const baseProps = {
  isOpen: true,
  onClose: vi.fn(),
  appUpdate: null,
  ollamaBinaryUpdate: null,
  ollamaUpdates: [],
  forecastDevUpdates: [],
  pulling: null,
  ollamaBinaryUpdating: false,
  ollamaBinaryPercent: 0,
  appDownloading: false,
  appPercent: 0,
  onPullModel: vi.fn(),
  onDownloadApp: vi.fn(),
  onUpdateOllamaBinary: vi.fn(),
  anchorLeft: 0,
};

describe("UpdateNotifications", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("garde les updates Ollama compactes", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        ollamaUpdates={[{ fullName: "llama3:latest", family: "llama3", tag: "latest" }]}
      />,
    );

    expect(screen.getByText("llama3:latest")).toBeTruthy();
    expect(screen.queryByLabelText("updates.showDetails")).toBeNull();
  });

  it("déplie et replie les notes de l'update app", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        appUpdate={{
          version: "0.9.4",
          assetUrl: "https://example.invalid/app.dmg",
          notesByLocale: {
            en: ["Context details."],
            fr: ["Détails du contexte."],
          },
        }}
      />,
    );

    const toggle = screen.getByLabelText("updates.showDetails");
    expect(toggle).toHaveAttribute("aria-expanded", "false");

    fireEvent.click(toggle);
    expect(screen.getByLabelText("updates.hideDetails")).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("Détails du contexte.")).toBeTruthy();

    fireEvent.click(screen.getByLabelText("updates.hideDetails"));
    expect(screen.getByLabelText("updates.showDetails")).toHaveAttribute("aria-expanded", "false");
  });

  it("masque la flèche si l'update app n'a pas de notes", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        appUpdate={{
          version: "0.9.4",
          assetUrl: "https://example.invalid/app.dmg",
          notesByLocale: null,
        }}
      />,
    );

    expect(screen.getByText("Beaver")).toBeTruthy();
    expect(screen.queryByLabelText("updates.showDetails")).toBeNull();
  });

  it("affiche une mise à jour Forecast uniquement comme information dev", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        forecastDevUpdates={[{
          id: "chronos",
          displayName: "Chronos",
          kind: "runtime",
          current: "2.3.1",
          latest: "2.4.0",
          sourceUrl: "https://pypi.org/project/chronos-forecasting/",
        }]}
      />,
    );

    expect(screen.getByText("Chronos")).toBeTruthy();
    expect(screen.getByText("updates.forecastDevRuntime · 2.3.1 → 2.4.0")).toBeTruthy();
    expect(screen.getByText("updates.forecastDevReview")).toBeTruthy();
  });

  it("garde aussi les commits des moteurs Forecast compacts", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        forecastDevUpdates={[{
          id: "kairos-engine",
          displayName: "Kairos engine",
          kind: "runtime",
          current: "0322393840ccf6e2bfe9c663f9dcd088a5a7ee07",
          latest: "abcdef1234567890abcdef1234567890abcdef12",
          sourceUrl: "https://github.com/foundation-model-research/Kairos",
        }]}
      />,
    );

    expect(screen.getByText("updates.forecastDevRuntime · 0322393 → abcdef1")).toBeTruthy();
  });
});
