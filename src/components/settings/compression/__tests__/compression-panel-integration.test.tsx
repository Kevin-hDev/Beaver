import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import { compressionProfileFixture } from "@/test-utils/compression-profile-fixture";
import type { CompressionProfile } from "@/types/compression-profile.generated";
import { CompressionPanel } from "../compression-panel";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: { name?: string }) => values?.name ? `${key}:${values.name}` : key,
  }),
}));
vi.mock("@/hooks/use-available-models", () => ({
  useAvailableModels: () => ({ groups: new Map(), loading: false }),
  withoutInteractiveOnlyModels: (groups: Map<string, unknown>) => groups,
}));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve({
    context_window: 64_000,
    band: "compact",
    system_tools_tokens: 12_000,
    summary_tokens: 4_000,
    categories_tokens: 20_000,
    reserve_tokens: 4_000,
    total_tokens: 40_000,
    projected_percent: 63,
    exceeds_window: false,
    high_risk: false,
  })),
}));

function api(overrides: Partial<CompressionProfilesController> = {}): CompressionProfilesController {
  const beaver = compressionProfileFixture();
  const custom: CompressionProfile = { ...compressionProfileFixture(), id: "custom", name: "Custom" };
  return {
    view: { automatic_enabled: true, global_profile_id: "custom", global_selection_revision: 1, profiles: [beaver, custom] },
    busy: false,
    setAutomaticEnabled: vi.fn(() => Promise.resolve(true)),
    selectGlobal: vi.fn(() => Promise.resolve(true)),
    save: vi.fn(() => Promise.resolve(true)),
    create: vi.fn(() => Promise.resolve(true)),
    rename: vi.fn(() => Promise.resolve(true)),
    resetBeaver: vi.fn(() => Promise.resolve(true)),
    deleteProfile: vi.fn(() => Promise.resolve(null)),
    undoDelete: vi.fn(() => Promise.resolve(true)),
    refresh: vi.fn(() => Promise.resolve()),
    ...overrides,
  };
}

describe("CompressionPanel integration", () => {
  it("enchaîne plage sous 64K, création et renommage sans validation implicite", async () => {
    const save = vi.fn((_profile: CompressionProfile) => Promise.resolve(true));
    const create = vi.fn(() => Promise.resolve(true));
    const rename = vi.fn(() => Promise.resolve(true));
    const controller = api({ save, create, rename });
    render(<CompressionPanel controller={controller} currentWindow={32_000} onClose={vi.fn()} />);
    await screen.findByText("settings.advanced.compressionProjectionTotal");

    fireEvent.click(screen.getByRole("tab", { name: /settings\.advanced\.compressionRange\.under_64k/ }));
    expect(screen.getByLabelText("settings.advanced.compressionAutomaticThreshold")).toBeDisabled();
    fireEvent.click(screen.getByRole("switch", { name: "settings.advanced.compressionUnder64Title" }));
    expect(save).toHaveBeenCalledWith(expect.objectContaining({ allow_under_64k: true }));

    fireEvent.click(screen.getByRole("button", { name: "settings.advanced.compressionNewProfile" }));
    expect(screen.getByText("settings.advanced.compressionCreateFrom:Custom")).toBeInTheDocument();
    const cancel = screen.getAllByRole("button", {
      name: "settings.advanced.compressionCancel",
    }).find((button) => button.getAttribute("tabindex") !== "-1");
    if (!cancel) throw new Error("cancel button missing");
    fireEvent.click(cancel);
    expect(create).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "settings.advanced.compressionRename" }));
    fireEvent.change(screen.getByRole("textbox", { name: "settings.advanced.compressionProfileName" }), {
      target: { value: "Changed" },
    });
    fireEvent.mouseDown(screen.getByRole("dialog", { name: "settings.advanced.compressionPanelTitle" }));
    expect(rename).not.toHaveBeenCalled();
  });

  it("garde la création ouverte après une erreur générique du backend", async () => {
    const create = vi.fn(() => Promise.resolve(false));
    render(<CompressionPanel controller={api({ create })} currentWindow={128_000} onClose={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "settings.advanced.compressionNewProfile" }));
    const input = screen.getByRole("textbox", { name: "settings.advanced.compressionProfileName" });
    fireEvent.change(input, { target: { value: "Valid name" } });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() => expect(create).toHaveBeenCalledWith("custom", "Valid name"));
    expect(screen.getByRole("textbox", { name: "settings.advanced.compressionProfileName" })).toBeInTheDocument();
  });
});
