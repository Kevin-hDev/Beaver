import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import type { CompressionProfile } from "@/types/compression-profile.generated";
import { CompressionPanel } from "../compression-panel";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("../compression-profile-editor", () => ({
  CompressionProfileEditor: () => <div data-testid="compression-editor" />,
}));

const api = {
  view: {
    automatic_enabled: true,
    global_profile_id: "beaver",
    global_selection_revision: 1,
    profiles: [{ id: "beaver", name: "Beaver", revision: 1 } as CompressionProfile],
  },
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
} satisfies CompressionProfilesController;

describe("CompressionPanel", () => {
  it("se ferme par Échap et par le fond", () => {
    const close = vi.fn();
    const { rerender } = render(
      <CompressionPanel controller={api} currentWindow={128_000} onClose={close} />,
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(close).toHaveBeenCalledOnce();
    close.mockClear();
    rerender(<CompressionPanel controller={api} currentWindow={128_000} onClose={close} />);
    fireEvent.click(screen.getAllByRole("button", { name: "settings.advanced.compressionClose" })[0]);
    expect(close).toHaveBeenCalledOnce();
  });

  it("Échap ferme d'abord la création sans fermer le panneau", () => {
    const close = vi.fn();
    render(<CompressionPanel controller={api} currentWindow={128_000} onClose={close} />);
    fireEvent.click(screen.getByRole("button", { name: "settings.advanced.compressionNewProfile" }));
    expect(screen.getAllByRole("dialog")).toHaveLength(2);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.getAllByRole("dialog")).toHaveLength(1);
    expect(close).not.toHaveBeenCalled();
  });
});
