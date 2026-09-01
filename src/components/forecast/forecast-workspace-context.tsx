import { createContext, useContext } from "react";

const ForecastWorkspaceContext = createContext<string | null>(null);

export const ForecastWorkspaceProvider = ForecastWorkspaceContext.Provider;

export function useForecastSessionId(): string | null {
  return useContext(ForecastWorkspaceContext);
}
