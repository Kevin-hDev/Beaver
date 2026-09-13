import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { UpdateOperationSnapshot } from "@/types/update-progress.generated";
import { UpdateProgressWindow } from "./update-progress-window";

const mocks = vi.hoisted(() => ({
  close: vi.fn(),
  invoke: vi.fn(),
  listen: vi.fn(),
  operations: [] as UpdateOperationSnapshot[],
  resize: undefined as (() => void) | undefined,
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ close: mocks.close }) }));
vi.mock("./use-update-operations", () => ({
  useUpdateOperations: () => ({
    operations: mocks.operations,
    dismiss: (id: string): Promise<void> => {
      mocks.invoke("dismiss_update_operation", { id });
      return Promise.resolve();
    },
  }),
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string, values?: { count?: number; position?: number }) =>
    values?.count ? `${values.count} mises à jour` : values?.position ? `position ${values.position}` : key }),
}));

const base: UpdateOperationSnapshot = {
  id: "one", sequence: 1, kind: "app-release", label: "Beaver 1.4.2",
  status: "running", phase: "downloading", progressMode: "determinate",
  percent: 43, queuePosition: null, canCancel: true, canRetry: false, errorKey: null,
};

describe("UpdateProgressWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.invoke.mockResolvedValue(undefined);
    mocks.listen.mockResolvedValue(() => {});
    localStorage.clear();
    Object.defineProperty(window, "matchMedia", {
      configurable: true,
      value: vi.fn(() => ({ matches: false })),
    });
    mocks.resize = undefined;
    class Observer {
      constructor(callback: () => void) { mocks.resize = callback; }
      observe() {}
      disconnect() {}
    }
    vi.stubGlobal("ResizeObserver", Observer);
  });

  it("affiche quatre opérations simultanées sans inventer de pourcentage", () => {
    mocks.operations = [
      base,
      { ...base, id: "two", label: "Ollama", progressMode: "indeterminate", percent: null },
      { ...base, id: "three", label: "TimesFM", status: "queued", phase: "waiting", progressMode: "none", percent: null, queuePosition: 2, canCancel: false },
      { ...base, id: "four", label: "nomic", status: "cancelling", canCancel: false },
    ];
    render(<UpdateProgressWindow />);

    expect(screen.getAllByRole("listitem")).toHaveLength(4);
    expect(screen.getByText("4 mises à jour")).toBeTruthy();
    expect(screen.getByText(/43\s%/)).toBeTruthy();
    expect(screen.getByText("position 2")).toBeTruthy();
    expect(screen.queryByText(/0\s%/)).toBeNull();
    expect(screen.getAllByRole("button", { name: "updates.window.cancel" })).toHaveLength(2);
  });

  it("offre réessai et retrait après un échec générique", () => {
    mocks.operations = [{ ...base, status: "failed", canCancel: false, canRetry: true, errorKey: "private-path" }];
    render(<UpdateProgressWindow />);

    expect(screen.getByText("updates.window.failed")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "updates.window.retry" }));
    fireEvent.click(screen.getByRole("button", { name: "updates.window.remove" }));
    expect(mocks.invoke).toHaveBeenCalledWith("request_update_operation_retry", { id: "one" });
    expect(mocks.invoke).toHaveBeenCalledWith("dismiss_update_operation", { id: "one" });
  });

  it("valide le thème initial et les changements du canal fermé", () => {
    localStorage.setItem("clgo-theme", "extension:absente");
    localStorage.setItem("clgo-theme-base", "dark");
    mocks.operations = [base];
    render(<UpdateProgressWindow />);
    expect(document.documentElement.dataset.palette).toBe("dark");

    const call = mocks.listen.mock.calls.find(([name]) => name === "beaver-theme-changed");
    const listener = call?.[1] as ((event: { payload: unknown }) => void) | undefined;
    listener?.({ payload: { palette: "cobalt-frost", colorScheme: "light" } });
    expect(document.documentElement.dataset.palette).toBe("cobalt-frost");
    listener?.({ payload: { palette: "unknown", colorScheme: "dark" } });
    expect(document.documentElement.dataset.palette).toBe("cobalt-frost");
  });

  it("redimensionne uniquement quand la hauteur change et ferme sans annuler", () => {
    const height = vi.spyOn(HTMLElement.prototype, "scrollHeight", "get").mockReturnValue(200);
    mocks.operations = [base];
    render(<UpdateProgressWindow />);
    expect(mocks.invoke).toHaveBeenCalledWith("resize_update_progress_window", { height: 200 });

    mocks.resize?.();
    expect(mocks.invoke.mock.calls.filter(([name]) => name === "resize_update_progress_window")).toHaveLength(1);
    height.mockReturnValue(201);
    mocks.resize?.();
    expect(mocks.invoke).toHaveBeenCalledWith("resize_update_progress_window", { height: 201 });

    fireEvent.click(screen.getByRole("button", { name: "updates.window.close" }));
    expect(mocks.close).toHaveBeenCalledOnce();
    expect(mocks.invoke.mock.calls.some(([name]) => String(name).startsWith("cancel_"))).toBe(false);
    height.mockRestore();
  });
});
