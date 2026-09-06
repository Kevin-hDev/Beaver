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
  onDismissUpdate: vi.fn(),
  onCancelApp: vi.fn(),
  onCancelOllamaBinary: vi.fn(),
  onCancelModel: vi.fn(),
  appCancelling: false,
  ollamaBinaryCancelling: false,
  modelCancelling: false,
  anchorRef: { current: null },
};

describe("UpdateNotifications", () => {
  it("positions a portal at its actual anchor and returns focus on Escape without cancelling", () => {
    const anchor = document.createElement("button");
    document.body.appendChild(anchor);
    anchor.getBoundingClientRect = () => ({ left: 25, right: 45, top: 15, bottom: 35, width: 20, height: 20, x: 25, y: 15, toJSON: () => ({}) });
    const close = vi.fn();
    const cancel = vi.fn();
    const { container } = render(<UpdateNotifications {...baseProps} anchorRef={{ current: anchor }} onClose={close} onCancelApp={cancel}>
      <button type="button">Continuer l’installation</button>
    </UpdateNotifications>);
    const region = screen.getByRole("region", { name: "extensionInstalls.title" });
    expect(container.contains(region)).toBe(false);
    expect(region.style.left).toBe("25px");
    expect(region.style.top).toBe("39px");
    expect(region.style.maxWidth).toBe(""); // Preserve the shared CSS width cap.
    expect(screen.getByRole("button", { name: "Continuer l’installation" })).toHaveFocus();
    fireEvent.keyDown(window, { code: "Escape" });
    expect(close).toHaveBeenCalledOnce();
    expect(anchor).toHaveFocus();
    expect(cancel).not.toHaveBeenCalled();
    anchor.remove();
  });
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("garde les updates Ollama compactes", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        ollamaUpdates={[{ fullName: "llama3:latest", family: "llama3", tag: "latest", latestDigest: "abc123" }]}
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

  it("masque une version précise avec la croix au survol", () => {
    const onDismissUpdate = vi.fn();
    render(
      <UpdateNotifications
        {...baseProps}
        onDismissUpdate={onDismissUpdate}
        appUpdate={{ version: "0.9.4", assetUrl: "https://example.invalid/app.dmg" }}
      />,
    );

    fireEvent.click(screen.getByLabelText("updates.dismiss"));
    expect(onDismissUpdate).toHaveBeenCalledWith({
      kind: "app",
      subject: "beaver",
      version: "0.9.4",
    });
  });

  it("ne permet jamais de masquer les informations Forecast dev", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        forecastDevUpdates={[{
          id: "chronos",
          displayName: "Chronos",
          kind: "runtime",
          current: "2.3.1",
          latest: "2.4.0",
          sourceUrl: "https://example.invalid/chronos",
        }]}
      />,
    );

    expect(screen.queryByLabelText("updates.dismiss")).toBeNull();
  });

  it("remplace la croix par Annuler pendant un téléchargement", () => {
    const onCancelApp = vi.fn();
    render(
      <UpdateNotifications
        {...baseProps}
        appUpdate={{ version: "0.9.4", assetUrl: "https://example.invalid/app.dmg" }}
        appDownloading
        appPercent={42}
        onCancelApp={onCancelApp}
      />,
    );

    expect(screen.queryByLabelText("updates.dismiss")).toBeNull();
    fireEvent.click(screen.getByText("common.cancel"));
    expect(onCancelApp).toHaveBeenCalledOnce();
  });
});
