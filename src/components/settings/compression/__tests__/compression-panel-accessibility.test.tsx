import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import type { CompressionProfile } from "@/types/compression-profile.generated";
import { CompressionPanel } from "../compression-panel";
import { compressionProfilesViewFixture } from "@/test-utils/compression-profile-fixture";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("../compression-profile-editor", () => ({
  CompressionProfileEditor: () => <button type="button">last-focusable</button>,
}));

const profile = { id: "beaver", name: "Beaver", revision: 1 } as CompressionProfile;
const controller: CompressionProfilesController = {
  view: compressionProfilesViewFixture([profile]),
  busy: false,
  setAutomaticEnabled: vi.fn(() => Promise.resolve(true)),
  selectGlobal: vi.fn(() => Promise.resolve(true)),
  save: vi.fn(() => Promise.resolve(true)),
  create: vi.fn(() => Promise.resolve(true)),
  rename: vi.fn(() => Promise.resolve(true)),
  resetBeaver: vi.fn(() => Promise.resolve(null)),
  resetPrompts: vi.fn(() => Promise.resolve(true)),
  deleteProfile: vi.fn(() => Promise.resolve(null)),
  undoDelete: vi.fn(() => Promise.resolve(true)),
  refresh: vi.fn(() => Promise.resolve()),
};

describe("CompressionPanel accessibility", () => {
  it("nomme le dialogue, garde le focus dedans et exclut le fond du clavier", () => {
    render(<CompressionPanel controller={controller} currentWindow={128_000} onClose={vi.fn()} />);
    const dialog = screen.getByRole("dialog", { name: "settings.advanced.compressionPanelTitle" });
    const closeButtons = screen.getAllByRole("button", { name: "settings.advanced.compressionClose" });
    const close = closeButtons.find((button) => button.getAttribute("tabindex") !== "-1");
    const backdrop = closeButtons.find((button) => button.getAttribute("tabindex") === "-1");
    if (!close || !backdrop) throw new Error("dialog controls missing");

    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(close).toHaveFocus();
    expect(backdrop).toHaveAttribute("tabindex", "-1");
    fireEvent.keyDown(window, { key: "Tab", shiftKey: true });
    expect(screen.getByRole("button", { name: "last-focusable" })).toHaveFocus();
    fireEvent.keyDown(window, { key: "Tab" });
    expect(close).toHaveFocus();
  });

  it("place le focus dans le nom du profil et ferme seulement la couche supérieure", () => {
    const close = vi.fn();
    render(<CompressionPanel controller={controller} currentWindow={128_000} onClose={close} />);
    fireEvent.click(screen.getByRole("button", { name: "settings.advanced.compressionNewProfile" }));
    const input = screen.getByRole("textbox", { name: "settings.advanced.compressionProfileName" });
    expect(input).toHaveFocus();
    expect(screen.getAllByRole("dialog")).toHaveLength(2);

    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.getAllByRole("dialog")).toHaveLength(1);
    expect(close).not.toHaveBeenCalled();
  });
});
