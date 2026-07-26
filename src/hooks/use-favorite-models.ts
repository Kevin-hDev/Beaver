import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useFsEvent } from "@/hooks/use-fs-event";

export interface FavoriteModel {
  provider: string;
  model: string;
}

export function useFavoriteModels() {
  const [favorites, setFavorites] = useState<FavoriteModel[]>([]);

  const refresh = useCallback(async () => {
    try {
      const list = await invoke<FavoriteModel[]>("list_favorite_models");
      setFavorites(list);
    } catch {
      setFavorites([]);
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
  }, [refresh]);

  useFsEvent("fs:config-changed", () => { void refresh(); });

  const isFavorite = useCallback(
    (provider: string, model: string) =>
      favorites.some((f) => f.provider === provider && f.model === model),
    [favorites]
  );

  const toggle = useCallback(
    async (provider: string, model: string) => {
      if (isFavorite(provider, model)) {
        await invoke("remove_favorite_model", { provider, model });
      } else {
        await invoke("add_favorite_model", { provider, model });
      }
      await refresh();
    },
    [isFavorite, refresh]
  );

  return { favorites, isFavorite, toggle, refresh };
}
