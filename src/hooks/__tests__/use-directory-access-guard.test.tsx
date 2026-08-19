import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import { describe, expect, it, vi } from "vitest";
import { AppNavigationActionsProvider } from "../use-app-navigation-actions";
import { useDirectoryAccessGuard } from "../use-directory-access-guard";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

describe("useDirectoryAccessGuard", () => {
  it("conserve la sélection précédente et propose les réglages quand le dossier est refusé", async () => {
    vi.mocked(invoke).mockResolvedValue({
      allowed: false,
      allowed_paths: ["/project/allowed"],
    });
    const openSettings = vi.fn();
    const onAllowed = vi.fn();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <AppNavigationActionsProvider openFileAccessSettings={openSettings}>
        {children}
      </AppNavigationActionsProvider>
    );
    const { result } = renderHook(() => useDirectoryAccessGuard(), { wrapper });

    await act(async () => {
      await result.current.request("/project", onAllowed);
    });

    expect(onAllowed).not.toHaveBeenCalled();
    expect(result.current.prompt?.allowedPaths).toEqual(["/project/allowed"]);
    act(() => result.current.prompt?.onSettings());
    expect(openSettings).toHaveBeenCalledOnce();
    expect(result.current.prompt).toBeUndefined();
  });

  it("valide la sélection uniquement après l’accord du backend", async () => {
    vi.mocked(invoke).mockResolvedValue({ allowed: true, allowed_paths: ["/"] });
    const onAllowed = vi.fn();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <AppNavigationActionsProvider openFileAccessSettings={vi.fn()}>
        {children}
      </AppNavigationActionsProvider>
    );
    const { result } = renderHook(() => useDirectoryAccessGuard(), { wrapper });

    void result.current.request("/project", onAllowed);

    await waitFor(() => expect(onAllowed).toHaveBeenCalledOnce());
  });

  it("réaffiche le blocage si les réglages changent pendant la sélection", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ allowed: true, allowed_paths: ["/"] })
      .mockResolvedValueOnce({ allowed: false, allowed_paths: ["/project/allowed"] });
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <AppNavigationActionsProvider openFileAccessSettings={vi.fn()}>
        {children}
      </AppNavigationActionsProvider>
    );
    const { result } = renderHook(() => useDirectoryAccessGuard(), { wrapper });

    await act(async () => {
      await result.current.request("/project", () => Promise.reject(new Error("changed")));
    });

    expect(result.current.prompt?.allowedPaths).toEqual(["/project/allowed"]);
  });

  it("traduit une panne du stockage projet après une autorisation valide", async () => {
    vi.mocked(invoke).mockResolvedValue({ allowed: true, allowed_paths: ["/"] });
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <AppNavigationActionsProvider openFileAccessSettings={vi.fn()}>
        {children}
      </AppNavigationActionsProvider>
    );
    const { result } = renderHook(() => useDirectoryAccessGuard(), { wrapper });

    await act(async () => {
      await result.current.request(
        "/project",
        () => Promise.reject(new Error("project-store-unavailable")),
      );
    });

    expect(showToast).toHaveBeenCalledWith("errors.localStore.projects", "error");
  });
});
