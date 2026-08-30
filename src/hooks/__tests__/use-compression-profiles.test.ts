import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import type {
  CompressionProfile,
  CompressionProfilesView,
} from "@/types/compression-profile.generated";

const { fsCallbacks, showToast } = vi.hoisted(() => ({
  fsCallbacks: new Map<string, () => void>(),
  showToast: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@/hooks/use-fs-event", () => ({
  useFsEvent: (event: string, callback: () => void) => fsCallbacks.set(event, callback),
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast }));

import { useCompressionProfiles } from "@/hooks/use-compression-profiles";

const profile = (id: string, threshold: number, revision = 1) => ({
  id,
  name: id === "beaver" ? "Beaver" : "Custom",
  revision,
  threshold_percent: threshold,
}) as CompressionProfile;

const view = (threshold = 90, revision = 1): CompressionProfilesView => ({
  automatic_enabled: true,
  global_profile_id: "beaver",
  global_selection_revision: 1,
  profiles: [profile("custom", 80), profile("beaver", threshold, revision)],
});

beforeEach(() => {
  vi.clearAllMocks();
  fsCallbacks.clear();
});

describe("useCompressionProfiles", () => {
  it("charge et ordonne Beaver avant les profils personnalisés", async () => {
    vi.mocked(invoke).mockResolvedValue(view());

    const { result } = renderHook(() => useCompressionProfiles());

    await waitFor(() => expect(result.current.view).not.toBeNull());
    expect(result.current.view?.profiles.map((item) => item.id)).toEqual(["beaver", "custom"]);
    expect(invoke).toHaveBeenCalledWith("get_compression_profiles");
  });

  it("rafraîchit depuis l'événement persistant sans vider la vue confirmée", async () => {
    let resolveRefresh: ((value: CompressionProfilesView) => void) | undefined;
    vi.mocked(invoke)
      .mockResolvedValueOnce(view())
      .mockImplementationOnce(() => new Promise((resolve) => { resolveRefresh = resolve; }));
    const { result } = renderHook(() => useCompressionProfiles());
    await waitFor(() => expect(result.current.view).not.toBeNull());

    act(() => fsCallbacks.get("fs:compression-profiles-changed")?.());

    expect(result.current.view?.profiles[0].threshold_percent).toBe(90);
    act(() => resolveRefresh?.(view(82, 2)));
    await waitFor(() => expect(result.current.view?.profiles[0].threshold_percent).toBe(82));
  });

  it("borne les sauvegardes à une mutation et une dernière valeur en attente", async () => {
    let resolveFirst: ((value: CompressionProfilesView) => void) | undefined;
    vi.mocked(invoke)
      .mockResolvedValueOnce(view())
      .mockImplementationOnce(() => new Promise((resolve) => { resolveFirst = resolve; }))
      .mockResolvedValueOnce(view(88, 3));
    const { result } = renderHook(() => useCompressionProfiles());
    await waitFor(() => expect(result.current.view).not.toBeNull());
    const beaver = result.current.view!.profiles[0];

    let last: Promise<boolean> | undefined;
    act(() => {
      void result.current.save({ ...beaver, threshold_percent: 89 });
      void result.current.save({ ...beaver, threshold_percent: 87 });
      last = result.current.save({ ...beaver, threshold_percent: 88 });
    });
    expect(result.current.busy).toBe(false);
    act(() => resolveFirst?.(view(89, 2)));
    await expect(last).resolves.toBe(true);

    const saves = vi.mocked(invoke).mock.calls.filter(([command]) => command === "save_compression_profile");
    expect(saves).toHaveLength(2);
    expect(saves[1][1]).toMatchObject({ input: { revision: 2, threshold_percent: 88 } });
  });

  it("persiste l'interrupteur automatique sans modifier le profil", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(view()).mockResolvedValueOnce({
      ...view(),
      automatic_enabled: false,
    });
    const { result } = renderHook(() => useCompressionProfiles());
    await waitFor(() => expect(result.current.view).not.toBeNull());

    await act(async () => {
      expect(await result.current.setAutomaticEnabled(false)).toBe(true);
    });

    expect(invoke).toHaveBeenLastCalledWith(
      "set_automatic_compression_enabled",
      { enabled: false },
    );
    expect(result.current.view?.automatic_enabled).toBe(false);
  });

  it("réinitialise les prompts du profil actif via l'autorité backend", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(view()).mockResolvedValueOnce(view(90, 2));
    const { result } = renderHook(() => useCompressionProfiles());
    await waitFor(() => expect(result.current.view).not.toBeNull());

    await act(async () => {
      expect(await result.current.resetPrompts("beaver")).toBe(true);
    });

    expect(invoke).toHaveBeenLastCalledWith(
      "reset_compression_profile_prompts",
      { profileId: "beaver" },
    );
    expect(result.current.view?.profiles[0].revision).toBe(2);
  });

  it("conserve la dernière vue confirmée et affiche une erreur générique", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(view()).mockRejectedValueOnce(new Error("secret"));
    const { result } = renderHook(() => useCompressionProfiles());
    await waitFor(() => expect(result.current.view).not.toBeNull());

    await act(async () => {
      expect(await result.current.selectGlobal("custom")).toBe(false);
    });

    expect(result.current.view?.global_profile_id).toBe("beaver");
    expect(showToast).toHaveBeenCalledOnce();
    expect(showToast.mock.calls[0][0]).not.toContain("secret");
  });
});
