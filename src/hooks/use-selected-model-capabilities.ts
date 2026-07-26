import { useMemo } from "react";
import { useAvailableModels } from "@/hooks/use-available-models";

export function useSelectedModelCapabilities(provider: string, model: string) {
  const { groups } = useAvailableModels();
  return useMemo(
    () => groups.get(provider)?.find((entry) => entry.id === model) ?? null,
    [groups, provider, model],
  );
}
