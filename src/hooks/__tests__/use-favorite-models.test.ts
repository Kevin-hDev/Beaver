import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useFavoriteModels } from "../use-favorite-models";
import type { FavoriteModel } from "../use-favorite-models";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
vi.mock("@/hooks/use-fs-event", () => ({
  useFsEvent: vi.fn(),
}));

const mockFavorites: FavoriteModel[] = [
  { provider: "anthropic", model: "claude-3-opus" },
  { provider: "ollama", model: "llama3" },
];

describe("useFavoriteModels", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue(mockFavorites);
  });

  it("charge les favoris au mount via list_favorite_models", async () => {
    const { result } = renderHook(() => useFavoriteModels());

    await waitFor(() => {
      expect(result.current.favorites).toEqual(mockFavorites);
    });

    expect(invoke).toHaveBeenCalledWith("list_favorite_models");
  });

  it("isFavorite retourne true si provider + model match", async () => {
    const { result } = renderHook(() => useFavoriteModels());

    await waitFor(() => {
      expect(result.current.favorites).toHaveLength(2);
    });

    expect(result.current.isFavorite("anthropic", "claude-3-opus")).toBe(true);
  });

  it("isFavorite retourne false si pas de match", async () => {
    const { result } = renderHook(() => useFavoriteModels());

    await waitFor(() => {
      expect(result.current.favorites).toHaveLength(2);
    });

    expect(result.current.isFavorite("anthropic", "claude-3-haiku")).toBe(false);
    expect(result.current.isFavorite("openai", "gpt-4")).toBe(false);
  });

  it("toggle appelle add_favorite_model si le modèle n'est pas favori", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockFavorites) // list au mount
      .mockResolvedValueOnce(undefined)     // add_favorite_model
      .mockResolvedValueOnce([...mockFavorites, { provider: "openai", model: "gpt-4" }]); // refresh

    const { result } = renderHook(() => useFavoriteModels());

    await waitFor(() => expect(result.current.favorites).toHaveLength(2));

    await act(async () => {
      await result.current.toggle("openai", "gpt-4");
    });

    expect(invoke).toHaveBeenCalledWith("add_favorite_model", {
      provider: "openai",
      model: "gpt-4",
    });
    expect(result.current.favorites).toHaveLength(3);
  });

  it("toggle appelle remove_favorite_model si le modèle est déjà favori", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockFavorites) // list au mount
      .mockResolvedValueOnce(undefined)     // remove_favorite_model
      .mockResolvedValueOnce([{ provider: "ollama", model: "llama3" }]); // refresh sans anthropic

    const { result } = renderHook(() => useFavoriteModels());

    await waitFor(() => expect(result.current.favorites).toHaveLength(2));

    await act(async () => {
      await result.current.toggle("anthropic", "claude-3-opus");
    });

    expect(invoke).toHaveBeenCalledWith("remove_favorite_model", {
      provider: "anthropic",
      model: "claude-3-opus",
    });
    expect(result.current.favorites).toHaveLength(1);
  });

  it("toggle qui échoue (add_favorite_model rejette) remonte l'erreur", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockFavorites)           // list au mount
      .mockRejectedValueOnce(new Error("backend KO")); // add_favorite_model échoue

    const { result } = renderHook(() => useFavoriteModels());

    await waitFor(() => expect(result.current.favorites).toHaveLength(2));

    await expect(
      act(() => result.current.toggle("openai", "gpt-4")),
    ).rejects.toThrow();
  });

  it("chargement initial qui échoue → favoris = []", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("backend KO"));

    const { result } = renderHook(() => useFavoriteModels());

    await waitFor(() => {
      expect(result.current.favorites).toEqual([]);
    });
  });

  it("isFavorite avec provider vide retourne false", async () => {
    const { result } = renderHook(() => useFavoriteModels());

    await waitFor(() => expect(result.current.favorites).toHaveLength(2));

    expect(result.current.isFavorite("", "claude-3-opus")).toBe(false);
  });
});
