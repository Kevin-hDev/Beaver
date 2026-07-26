import { afterEach, beforeEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";

// --- Polyfill localStorage (jsdom ne fournit pas --localstorage-file) --------
// Plusieurs composants persistent de l'état UI en localStorage. On fournit un
// store en mémoire simple, vidé entre les tests pour éviter les fuites.
const memStore = new Map<string, string>();
const localStoragePolyfill: Storage = {
  get length() {
    return memStore.size;
  },
  clear: () => memStore.clear(),
  getItem: (k) => (memStore.has(k) ? memStore.get(k)! : null),
  key: (i) => Array.from(memStore.keys())[i] ?? null,
  removeItem: (k) => {
    memStore.delete(k);
  },
  setItem: (k, v) => {
    memStore.set(k, String(v));
  },
};
Object.defineProperty(globalThis, "localStorage", {
  value: localStoragePolyfill,
  configurable: true,
});

// --- Mock global réinitialisable de l'IPC Tauri -----------------------------
// Tous les composants/hooks importent invoke/listen directement depuis
// @tauri-apps/api. Sans ce mock global, chaque test devrait redéclarer
// vi.mock(...) dans son fichier. Ici on fournit une implémentation par défaut
// (résout undefined / listen no-op) que chaque test peut surcharger via
// vi.mocked(invoke).mockImplementation(...).

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((path: string) => path),
  isTauri: () => true,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
}));

// --- Nettoyage DOM + état après chaque test ----------------------------------
beforeEach(() => {
  memStore.clear();
});

afterEach(() => {
  cleanup();
  memStore.clear();
});
