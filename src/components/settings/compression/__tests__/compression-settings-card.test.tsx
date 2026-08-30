import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CompressionProfile } from "@/types/compression-profile.generated";

const controller = vi.hoisted(() => ({
  busy: false,
  selectGlobal: vi.fn(() => Promise.resolve(true)),
  save: vi.fn(() => Promise.resolve(true)),
  refresh: vi.fn(() => Promise.resolve()),
  create: vi.fn(() => Promise.resolve(true)),
  rename: vi.fn(() => Promise.resolve(true)),
  resetBeaver: vi.fn(() => Promise.resolve(true)),
  deleteProfile: vi.fn(() => Promise.resolve(null)),
  undoDelete: vi.fn(() => Promise.resolve(true)),
  view: null as null | {
    global_profile_id: string;
    global_selection_revision: number;
    profiles: CompressionProfile[];
  },
}));
const context = vi.hoisted(() => ({ max: 128_000 }));

vi.mock("@/hooks/use-compression-profiles", () => ({
  useCompressionProfiles: () => controller,
}));
vi.mock("@/hooks/use-context-progress", () => ({
  useContextProgress: () => context,
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => ({
      "settings.advanced.compressionProfileTitle": "Profil de compression",
      "settings.advanced.compressionProfileDesc": "Configuration par défaut",
      "settings.advanced.compressionAdvancedTitle": "Configuration avancée",
      "settings.advanced.compressionAdvancedDesc": "Profil et contenu conservé",
      "settings.advanced.compressionAdvanced": "Avancé",
      "settings.advanced.compressionThresholdTitle": "Seuil de compression",
      "settings.advanced.compressionThresholdDesc": "Part de la fenêtre",
      "settings.advanced.compressionDisabledUnder64": "Compression désactivée pour les fenêtres de contexte inférieures à 64K.",
    })[key] ?? key,
  }),
}));

import { CompressionSettingsCard } from "../compression-settings-card";

const profile = (id: string, name: string, threshold: number, allowUnder64 = false) => ({
  id,
  name,
  revision: 1,
  threshold_percent: threshold,
  allow_under_64k: allowUnder64,
}) as CompressionProfile;

beforeEach(() => {
  vi.clearAllMocks();
  context.max = 128_000;
  controller.view = {
    global_profile_id: "beaver",
    global_selection_revision: 1,
    profiles: [profile("beaver", "Beaver", 90), profile("custom", "Custom", 82)],
  };
});

describe("CompressionSettingsCard", () => {
  it("affiche Beaver puis les profils personnalisés avec le panneau fonctionnel", () => {
    render(<CompressionSettingsCard defaultModel="ollama:qwen" />);

    fireEvent.click(screen.getByRole("button", { name: /Beaver/i }));
    const options = [...document.querySelectorAll(".ss-option")];
    expect(options.map((option) => option.textContent)).toEqual(["Beaver", "Custom"]);
    fireEvent.click(screen.getByRole("button", { name: "Avancé" }));
    expect(screen.getByRole("dialog", { name: /settings.advanced.compressionPanelTitle/i })).toBeInTheDocument();
  });

  it("borne le seuil entre 1 et 90 et sauvegarde le profil entier", () => {
    render(<CompressionSettingsCard defaultModel="ollama:qwen" />);
    const slider = screen.getByRole("slider", { name: /Seuil de compression/i });

    expect(slider).toHaveAttribute("min", "1");
    expect(slider).toHaveAttribute("max", "90");
    expect(slider).toHaveValue("90");
    fireEvent.change(slider, { target: { value: "84" } });

    expect(controller.save).toHaveBeenCalledWith(expect.objectContaining({
      id: "beaver",
      threshold_percent: 84,
    }));
  });

  it("explique la désactivation sous 64K sans masquer les réglages", () => {
    context.max = 32_000;
    render(<CompressionSettingsCard defaultModel="ollama:qwen" />);

    expect(screen.getByText(/inférieures à 64K/i)).toBeInTheDocument();
    expect(screen.getByRole("slider")).toBeInTheDocument();
  });
});
