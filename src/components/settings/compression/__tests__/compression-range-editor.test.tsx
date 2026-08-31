import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import type { CompressionProfile } from "@/types/compression-profile.generated";
import { CompressionProfileEditor } from "../compression-profile-editor";
import {
  compressionLimitsFixture,
  compressionProfileFixture,
} from "@/test-utils/compression-profile-fixture";

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
    resetPrompts: vi.fn(),
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
        limits={compressionLimitsFixture()}
        automaticEnabled
      />,
    );

    expect(screen.getAllByRole("tab")).toHaveLength(3);
    expect(screen.getByLabelText("settings.advanced.compressionActiveRange")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("tab", {
      name: "settings.advanced.compressionRange.under_64k",
    }));
    expect(screen.queryByText("settings.advanced.compressionUnder64Warning")).not.toBeInTheDocument();
    expect(screen.getByLabelText("settings.advanced.compressionAutomaticThreshold")).toBeEnabled();
    expect(screen.getAllByRole("textbox")).toHaveLength(2);
    expect(screen.getAllByRole("textbox").every((field) => !field.hasAttribute("disabled"))).toBe(true);

    fireEvent.click(screen.getByRole("switch", {
      name: "settings.advanced.compressionUnder64Title",
    }));
    expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ allow_under_64k: true }));
    expect(screen.getByText("settings.advanced.compressionUnder64Warning")).toHaveClass(
      "toast-warning",
    );
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
        limits={compressionLimitsFixture()}
        automaticEnabled
      />,
    );

    expect(screen.queryByText("settings.advanced.compressionUnder64Warning")).not.toBeInTheDocument();
  });

  it("copie la plage éditée et n'expose que les quantités simples", () => {
    const save = vi.fn((_profile: CompressionProfile) => Promise.resolve(true));
    render(
      <CompressionProfileEditor
        profile={{ ...compressionProfileFixture(), allow_under_64k: true }}
        currentWindow={96_000}
        controller={controller(save)}
        limits={compressionLimitsFixture()}
        automaticEnabled
      />,
    );
    fireEvent.click(screen.getByRole("button", {
      name: "settings.advanced.compressionCopyOtherRanges",
    }));
    const saved = save.mock.lastCall?.[0];
    if (!saved) throw new Error("missing saved profile");
    expect(saved.under_64k).toEqual(saved.compact);
    expect(saved.large).toEqual(saved.compact);
    expect(screen.getByLabelText("settings.advanced.compressionRecentMessages")).toBeInTheDocument();
    expect(screen.queryByText("settings.advanced.compressionBudgetMode.fixed")).not.toBeInTheDocument();
  });

  it("enregistre une taille de résumé complète seulement à la sortie du champ", () => {
    const save = vi.fn((_profile: CompressionProfile) => Promise.resolve(true));
    render(
      <CompressionProfileEditor
        profile={compressionProfileFixture()}
        currentWindow={96_000}
        controller={controller(save)}
        limits={compressionLimitsFixture()}
        automaticEnabled
      />,
    );
    const input = screen.getByLabelText("settings.advanced.compressionSummaryMaximum");

    fireEvent.change(input, { target: { value: "2" } });
    expect(input).toHaveValue(2);
    expect(save).not.toHaveBeenCalled();
    fireEvent.change(input, { target: { value: "2500" } });
    expect(save).not.toHaveBeenCalled();
    fireEvent.blur(input);

    const saved = save.mock.lastCall?.[0];
    if (!saved) throw new Error("missing saved profile");
    expect(saved.compact.summary_max_tokens).toBe(2500);
  });

  it("garde le focus pour saisir 75 avant d'enregistrer le seuil", () => {
    const save = vi.fn((_profile: CompressionProfile) => Promise.resolve(true));
    render(
      <CompressionProfileEditor
        profile={compressionProfileFixture()}
        currentWindow={96_000}
        controller={controller(save)}
        limits={compressionLimitsFixture()}
        automaticEnabled
      />,
    );
    const input = screen.getByLabelText("settings.advanced.compressionAutomaticThreshold");
    input.focus();

    fireEvent.change(input, { target: { value: "7" } });
    expect(input).toHaveFocus();
    expect(save).not.toHaveBeenCalled();
    fireEvent.change(input, { target: { value: "75" } });
    expect(input).toHaveFocus();
    expect(save).not.toHaveBeenCalled();
    fireEvent.blur(input);

    const saved = save.mock.lastCall?.[0];
    if (!saved) throw new Error("missing saved profile");
    expect(saved.threshold_percent).toBe(75);
  });

  it("utilise les bornes reçues du backend pour les quantités", () => {
    const limits = compressionLimitsFixture();
    limits.max_messages = 3;
    render(
      <CompressionProfileEditor
        profile={compressionProfileFixture()}
        currentWindow={96_000}
        controller={controller()}
        limits={limits}
        automaticEnabled
      />,
    );

    expect(screen.getByLabelText("settings.advanced.compressionRecentMessages"))
      .toHaveAttribute("max", "3");
  });
});
