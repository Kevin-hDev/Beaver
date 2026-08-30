import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import type { CompressionProfile } from "@/types/compression-profile.generated";
import { CompressionProfileEditor } from "../compression-profile-editor";
import { compressionProfileFixture } from "@/test-utils/compression-profile-fixture";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("@/hooks/use-available-models", () => ({
  useAvailableModels: () => ({ groups: new Map(), loading: false }),
  withoutInteractiveOnlyModels: (groups: Map<string, unknown>) => groups,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => new Promise(() => {})),
}));

function controller(
  save: CompressionProfilesController["save"] = vi.fn(() => Promise.resolve(true)),
): CompressionProfilesController {
  return {
    view: null,
    busy: false,
    setAutomaticEnabled: vi.fn(),
    selectGlobal: vi.fn(),
    save,
    create: vi.fn(),
    rename: vi.fn(),
    resetBeaver: vi.fn(),
    deleteProfile: vi.fn(),
    undoDelete: vi.fn(),
    refresh: vi.fn(),
  };
}

describe("CompressionProfileEditor", () => {
  it("sépare la plage active de la plage éditée et active sous 64K sans recalcul", () => {
    const save = vi.fn((_profile: CompressionProfile) => Promise.resolve(true));
    const profile = compressionProfileFixture();
    render(
      <CompressionProfileEditor
        profile={profile}
        currentWindow={96_000}
        controller={controller(save)}
      />,
    );

    expect(screen.getAllByRole("tab")).toHaveLength(3);
    expect(screen.getByLabelText("settings.advanced.compressionActiveRange")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("tab", {
      name: "settings.advanced.compressionRange.under_64k",
    }));
    expect(screen.queryByText("settings.advanced.compressionUnder64Warning")).not.toBeInTheDocument();
    expect(screen.getByLabelText("settings.advanced.compressionAutomaticThreshold")).toBeDisabled();

    fireEvent.click(screen.getByRole("switch", {
      name: "settings.advanced.compressionUnder64Title",
    }));
    expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ allow_under_64k: true }));
    expect(screen.getByText("settings.advanced.compressionUnder64Warning")).toBeInTheDocument();
    expect(screen.getByLabelText("settings.advanced.compressionAutomaticThreshold")).toBeEnabled();
    const saved = save.mock.lastCall?.[0];
    if (!saved) throw new Error("missing saved profile");
    expect(saved.under_64k).toEqual(profile.under_64k);
  });

  it("ne montre jamais l'avertissement sous 64K sur une autre plage", () => {
    render(
      <CompressionProfileEditor
        profile={{ ...compressionProfileFixture(), allow_under_64k: true }}
        currentWindow={96_000}
        controller={controller()}
      />,
    );

    expect(screen.queryByText("settings.advanced.compressionUnder64Warning")).not.toBeInTheDocument();
  });

  it("copie la plage éditée et expose les trois modes de budget", () => {
    const save = vi.fn((_profile: CompressionProfile) => Promise.resolve(true));
    render(
      <CompressionProfileEditor
        profile={{ ...compressionProfileFixture(), allow_under_64k: true }}
        currentWindow={96_000}
        controller={controller(save)}
      />,
    );
    fireEvent.click(screen.getByRole("button", {
      name: "settings.advanced.compressionCopyOtherRanges",
    }));
    const saved = save.mock.lastCall?.[0];
    if (!saved) throw new Error("missing saved profile");
    expect(saved.under_64k).toEqual(saved.compact);
    expect(saved.large).toEqual(saved.compact);
    expect(screen.getAllByText("settings.advanced.compressionBudgetMode.fixed").length).toBeGreaterThan(0);
  });
});
