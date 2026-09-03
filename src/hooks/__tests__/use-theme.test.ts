import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useTheme } from "@/hooks/use-theme";
import type {
  ExtensionThemeCatalog,
  ExtensionThemeEntry,
} from "@/features/extension-ui/themes/theme-catalog";

const showToast = vi.hoisted(() => vi.fn());

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({ show: showToast }),
}));

const mediaListeners = new Set<() => void>();
let prefersDark = false;

beforeEach(() => {
  localStorage.clear();
  showToast.mockClear();
  prefersDark = false;
  mediaListeners.clear();
  document.documentElement.removeAttribute("data-theme");
  document.documentElement.removeAttribute("data-palette");
  document.documentElement.removeAttribute("style");
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: vi.fn(() => ({
      get matches() { return prefersDark; },
      media: "(prefers-color-scheme: dark)",
      onchange: null,
      addEventListener: (_event: string, listener: () => void) => mediaListeners.add(listener),
      removeEventListener: (_event: string, listener: () => void) => mediaListeners.delete(listener),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
});

function extensionTheme(): ExtensionThemeEntry {
  return {
    choice: "extension:com.example.night",
    paletteId: "com.example.night",
    extensionId: "com.example",
    sourceName: "Example",
    label: "Night",
    colorScheme: "dark",
    tokens: { "--void": "#010203" },
  };
}

function catalog(entries: ExtensionThemeEntry[]): ExtensionThemeCatalog {
  return {
    ready: true,
    entries,
    byChoice: new Map(entries.map((entry) => [entry.choice, entry])),
  };
}

describe("useTheme", () => {
  it("restaure et applique Émeraude nocturne comme thème sombre", async () => {
    localStorage.setItem("clgo-theme", "emerald-night");

    const { result } = renderHook(() => useTheme());

    await waitFor(() => expect(result.current.theme).toBe("emerald-night"));
    expect(result.current.choice).toBe("emerald-night");
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(document.documentElement).toHaveAttribute("data-palette", "emerald-night");
  });

  it("retombe sur le système quand la valeur sauvegardée est inconnue", async () => {
    localStorage.setItem("clgo-theme", "unknown-theme");

    const { result } = renderHook(() => useTheme());

    await waitFor(() => expect(result.current.choice).toBe("system"));
    expect(result.current.theme).toBe("light");
    expect(document.documentElement).toHaveAttribute("data-palette", "light");
  });

  it("suit les changements du système uniquement en mode système", async () => {
    const { result } = renderHook(() => useTheme());
    await waitFor(() => expect(mediaListeners.size).toBe(1));

    act(() => {
      prefersDark = true;
      mediaListeners.forEach((listener) => listener());
    });

    expect(result.current.theme).toBe("dark");
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(document.documentElement).toHaveAttribute("data-palette", "dark");
  });

  it("permet de sélectionner Émeraude nocturne", async () => {
    const { result } = renderHook(() => useTheme());

    act(() => result.current.setTheme("emerald-night"));

    await waitFor(() => expect(result.current.theme).toBe("emerald-night"));
    expect(localStorage.getItem("clgo-theme")).toBe("emerald-night");
  });

  it("restaure et applique Cobalt givré comme thème clair", async () => {
    localStorage.setItem("clgo-theme", "cobalt-frost");

    const { result } = renderHook(() => useTheme());

    await waitFor(() => expect(result.current.theme).toBe("cobalt-frost"));
    expect(result.current.choice).toBe("cobalt-frost");
    expect(document.documentElement).toHaveAttribute("data-theme", "light");
    expect(document.documentElement).toHaveAttribute("data-palette", "cobalt-frost");
  });

  it("restaure et applique Brume astrale comme thème sombre", async () => {
    localStorage.setItem("clgo-theme", "astral-mist");

    const { result } = renderHook(() => useTheme());

    await waitFor(() => expect(result.current.theme).toBe("astral-mist"));
    expect(result.current.choice).toBe("astral-mist");
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(document.documentElement).toHaveAttribute("data-palette", "astral-mist");
  });

  it("restaure et applique Éclipse écarlate comme thème sombre", async () => {
    localStorage.setItem("clgo-theme", "crimson-eclipse");

    const { result } = renderHook(() => useTheme());

    await waitFor(() => expect(result.current.theme).toBe("crimson-eclipse"));
    expect(result.current.choice).toBe("crimson-eclipse");
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(document.documentElement).toHaveAttribute("data-palette", "crimson-eclipse");
  });

  it("attend le catalogue avant d'appliquer un thème d'extension sauvegardé", async () => {
    localStorage.setItem("clgo-theme", "extension:com.example.night");
    localStorage.setItem("clgo-theme-base", "dark");
    const themeEntry = extensionTheme();

    const { result } = renderHook(() => useTheme());
    expect(result.current.choice).toBe(themeEntry.choice);
    expect(result.current.theme).toBe("dark");
    expect(document.documentElement).not.toHaveAttribute("data-palette");

    act(() => result.current.setThemeCatalog(catalog([themeEntry])));

    await waitFor(() => expect(result.current.theme).toBe(themeEntry.choice));
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(document.documentElement).toHaveAttribute("data-palette", themeEntry.paletteId);
    expect(document.documentElement.style.getPropertyValue("--void")).toBe("#010203");
    expect(showToast).not.toHaveBeenCalled();
  });

  it("revient au système et avertit si le thème actif disparaît", async () => {
    localStorage.setItem("clgo-theme", "extension:com.example.night");
    localStorage.setItem("clgo-theme-base", "dark");
    const themeEntry = extensionTheme();
    const { result } = renderHook(() => useTheme());

    act(() => result.current.setThemeCatalog(catalog([themeEntry])));
    await waitFor(() => expect(result.current.theme).toBe(themeEntry.choice));
    act(() => result.current.setThemeCatalog(catalog([])));

    await waitFor(() => expect(result.current.choice).toBe("system"));
    expect(result.current.theme).toBe("light");
    expect(localStorage.getItem("clgo-theme")).toBe("system");
    expect(document.documentElement.style.getPropertyValue("--void")).toBe("");
    expect(showToast).toHaveBeenCalledWith(expect.any(String), "warning");
  });
});
